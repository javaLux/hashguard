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

const CAPACITY: usize = 64 * 1024;

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

    // Build a Spinner-Progress-Bar
    let spinner =
        ProgressBar::new_spinner().with_message("Calculate hash sum... this may take a while");

    // Define the spinner style
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&utils::BOUNCING_BAR)
            .template("{spinner:.white} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );

    // Set spinner tick every 100ms
    spinner.enable_steady_tick(Duration::from_millis(100));

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
        log::error!(
            "Failed to receive hash sum from Hash-Worker-Thread - Details: {:?}",
            e
        );
        anyhow::anyhow!("Failed to receive hash sum from Hash-Worker-Thread.")
    });

    // Ensure the spinner is finished and cleared
    spinner.finish_and_clear();

    // Ensure the thread is joined
    handle.join().map_err(|e| {
        log::error!("Failed to join Hash-Worker-Thread - Details: {:?}", e);
        anyhow::anyhow!("Failed to join Hash-Worker-Thread.")
    })?;

    result?
}

/// Computes a hash for the given file dependent on the used algorithm.
/// Includes file name (if needed) and the file content.
fn hash_file<P: AsRef<Path>>(file: P, algorithm: Algorithm, include_names: bool) -> Result<String> {
    let file_path = file.as_ref();
    let file = File::open(file_path)?;
    let mut reader = BufReader::with_capacity(CAPACITY, file);
    let mut hasher = Hasher::new(algorithm);

    // Add the file name to the hash
    if include_names {
        if let Some(file_name) = file_path.file_name() {
            hasher.update(file_name.to_string_lossy().as_bytes());
        }
    }

    let mut buf = [0u8; CAPACITY];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
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

    let mut buf = [0u8; CAPACITY];

    for entry in entries {
        let path = entry.path();
        if include_names {
            let relative_path = path.strip_prefix(dir.as_ref()).map_err(|err| {
                log::error!(
                    "Failed to strip prefix from path: {} - Details: {:?}",
                    utils::absolute_path_as_string(path),
                    err
                );
                anyhow::anyhow!(err)
            })?;

            hasher.update(relative_path.to_string_lossy().as_bytes());
        }

        if path.is_file() {
            let mut reader = BufReader::with_capacity(CAPACITY, File::open(path)?);
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
        }
    }

    Ok(hex::encode(hasher.finalize()))
}
