use crate::{
    hasher::{Algorithm, Hasher},
    utils,
};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::Duration,
};
use walkdir::WalkDir;

struct HashSpinner {
    spinner: ProgressBar,
    processed_bytes: usize,
}

impl HashSpinner {
    fn new() -> Self {
        let spinner = ProgressBar::new_spinner()
            .with_message("|Processed: 0 B| Calculate hash sum... this may take a while");
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&utils::BOUNCING_BAR)
                .template("{spinner:.white} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        spinner.enable_steady_tick(Duration::from_millis(100));
        HashSpinner {
            spinner,
            processed_bytes: 0,
        }
    }

    fn finish_and_clear(self) {
        self.spinner.finish_and_clear();
    }

    fn update(&mut self, bytes: usize) {
        self.processed_bytes += bytes;
        self.spinner.set_message(format!(
            "|Processed: {}| Calculate hash sum... this may take a while",
            utils::convert_bytes_to_human_readable(self.processed_bytes)
        ));
    }
}

pub fn get_buffer_hash(buffer: &[u8], algorithm: Algorithm) -> String {
    log::info!(
        "Try to calculate {} hash for a given byte buffer of size: {}",
        algorithm,
        utils::convert_bytes_to_human_readable(buffer.len())
    );

    Hasher::new(algorithm).digest_hex_lower(buffer)
}

/// Calculates the hash sum of the given data.
///
/// This function performs the following tasks:
/// - Spawns a new thread to calculate the hash sum.
/// - Displays a spinner to indicate progress.
/// - Ensures that all spawned threads are joined (completed) before returning the final result.
///
/// # Errors
/// - If the hash calculation fails, logs the error and returns a descriptive error message.
/// - If there is an issue with sending the hash result back to the main thread, an error will be
///   logged and returned.
///
/// This function is designed for multi-threaded environments where lengthy I/O or CPU-bound operations
/// benefit from non-blocking UI feedback (spinner) and graceful interruption handling.
pub fn get_hash_for_object(
    p: PathBuf,
    algorithm: Algorithm,
    include_names: bool,
) -> Result<String> {
    log::info!(
        "Try to calculate {} hash for {}: '{}'",
        algorithm,
        if p.is_dir() { "directory" } else { "file" },
        utils::absolute_path_as_string(&p)
    );

    // Use thread-safe Channels to transfer the Hash sum to the Main-Thread
    let (sender, receiver) = mpsc::channel();

    let handle = thread::Builder::new()
        .name("Hash-Worker-Thread".to_string())
        .spawn(move || {
            // Send the hash sum to the main thread
            let result = if p.is_dir() {
                hash_directory(p, algorithm, include_names)
            } else {
                hash_file(p, algorithm, include_names)
            };

            // Send the result back to the main thread
            sender
                .send(result)
                .expect("Failed to send hash sum to main thread");
        })
        .map_err(|e| {
            log::error!("Failed to spawn Hash-Worker-Thread - Details: {:?}", e);
            anyhow::anyhow!("Failed to spawn Hash-Worker-Thread.")
        })?;

    // Wait for the hash sum calculation to complete
    let result = receiver.recv().map_err(|e| {
        log::error!("Failed to receive hash sum from Hash-Worker-Thread - Details: {e:?}");
        anyhow::anyhow!("Failed to receive hash sum from Hash-Worker-Thread.")
    });

    // Ensure the thread is joined
    handle.join().map_err(|e| {
        log::error!("Failed to join Hash-Worker-Thread - Details: {e:?}");
        anyhow::anyhow!("Failed to join Hash-Worker-Thread.")
    })?;

    result?
}

/// Computes a hash for the given file dependent on the used algorithm.
/// Includes file name (if needed) and the file content.
fn hash_file<P: AsRef<Path>>(file: P, algorithm: Algorithm, include_names: bool) -> Result<String> {
    let file_path = file.as_ref();
    let file = File::open(file_path).map_err(|io_err| {
        let msg = format!(
            "Failed to open file: {}",
            utils::absolute_path_as_string(file_path),
        );
        log::error!("{msg} - Details: {io_err:?}");

        anyhow::anyhow!(msg)
    })?;
    let mut reader = BufReader::with_capacity(utils::CAPACITY, file);
    let mut hasher = Hasher::new(algorithm);
    let mut spinner = HashSpinner::new();

    // Add the file name to the hash
    if include_names {
        if let Some(file_name) = file_path.file_name() {
            hasher.update(file_name.to_string_lossy().as_bytes());
        }
    }

    let mut buf = [0u8; utils::CAPACITY];

    let result = loop {
        match reader.read(&mut buf) {
            Ok(n) => {
                if n == 0 {
                    break Ok(());
                }
                hasher.update(&buf[..n]);
                spinner.update(n);
            }
            Err(io_err) => {
                let msg = format!(
                    "Failed to read from file: {}",
                    utils::absolute_path_as_string(file_path),
                );
                log::error!("{msg} - Details: {io_err:?}");

                break Err(anyhow::anyhow!(msg));
            }
        }
    };

    spinner.finish_and_clear();
    result?;
    Ok(hex::encode(hasher.finalize()))
}

/// Computes a hash for the given directory dependent on the used algorithm.
/// Includes file and directory names (if needed) and the file contents.
fn hash_directory<P: AsRef<Path>>(
    dir: P,
    algorithm: Algorithm,
    include_names: bool,
) -> Result<String> {
    let root = dir.as_ref();
    let mut spinner = HashSpinner::new();

    let entries: Vec<_> = WalkDir::new(root)
        .sort_by_key(|e| e.path().to_path_buf()) // Sort entries to ensure deterministic hashing
        .max_depth(usize::MAX)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path() != root) // exclude the root directory
        .collect();

    let mut hasher = Hasher::new(algorithm);

    // Add the root directory name to the hash
    if include_names {
        if let Some(root_name) = root.file_name() {
            hasher.update(root_name.to_string_lossy().as_bytes());
        }
    }

    let mut result: Result<()> = Result::Ok(());

    let mut buf = [0u8; utils::CAPACITY];

    for entry in entries {
        let path = entry.path();
        if include_names {
            let relative_path = match path.strip_prefix(root) {
                Ok(relative_path) => relative_path,
                Err(err) => {
                    let msg = format!(
                        "Failed to strip prefix from path: {}",
                        utils::absolute_path_as_string(path),
                    );
                    log::error!("{msg} - Details: {err:?}");
                    result = Err(anyhow::anyhow!(msg));
                    break;
                }
            };

            hasher.update(relative_path.to_string_lossy().as_bytes());
        }

        if path.is_file() {
            match File::open(path) {
                Ok(file) => {
                    let mut reader = BufReader::with_capacity(utils::CAPACITY, file);
                    let r = loop {
                        match reader.read(&mut buf) {
                            Ok(n) => {
                                if n == 0 {
                                    break Ok(());
                                }
                                hasher.update(&buf[..n]);
                                spinner.update(n);
                            }
                            Err(io_err) => {
                                let msg = format!(
                                    "Failed to read from file: {}",
                                    utils::absolute_path_as_string(path),
                                );
                                log::error!("{msg} - Details: {io_err:?}");

                                break Err(anyhow::anyhow!(msg));
                            }
                        }
                    };
                    if let Err(io_err) = r {
                        result = Err(io_err);
                        break;
                    }
                }
                Err(io_err) => {
                    let msg = format!(
                        "Failed to open file: {}",
                        utils::absolute_path_as_string(path),
                    );
                    log::error!("{} - Details: {:?}", msg, io_err);
                    result = Err(anyhow::anyhow!(msg));
                    break;
                }
            }
        }
    }

    spinner.finish_and_clear();
    result?;
    Ok(hex::encode(hasher.finalize()))
}
