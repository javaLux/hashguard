use chksum::{chksum, Chksumable, Hash};
use clap::ValueEnum;
use indicatif::{ProgressBar, ProgressStyle};
use std::{path::PathBuf, thread, time::Duration};

use color_eyre::eyre::Result;

use crate::utils;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
/// Supported hash algorithm for calculating the hash sum
pub enum Algorithm {
    MD5,
    SHA1,
    SHA2_224,
    #[default]
    SHA2_256,
    SHA2_384,
    SHA2_512,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Algorithm::MD5 => write!(f, "MD5"),
            Algorithm::SHA1 => write!(f, "SHA1"),
            Algorithm::SHA2_224 => write!(f, "SHA2-224"),
            Algorithm::SHA2_256 => write!(f, "SHA2-256"),
            Algorithm::SHA2_384 => write!(f, "SHA2-384"),
            Algorithm::SHA2_512 => write!(f, "SHA2-512"),
        }
    }
}

/// Calculates the hash sum for a given file or directory using the specified hashing algorithm and returns
/// the result as a lowercase hexadecimal string.
pub fn get_file_hash(path: PathBuf, algorithm: Algorithm) -> Result<String> {
    log::debug!(
        "Try to calculate {} hash sum for file: '{}'",
        algorithm,
        utils::get_absolute_path(&path)
    );

    get_hash_sum_as_lower_hex(path, algorithm)
}

/// Calculates the hash sum for a given byte buffer using the specified hashing algorithm and returns
/// the result as a lowercase hexadecimal string.
pub fn get_buffer_hash(buffer: Vec<u8>, algorithm: Algorithm) -> Result<String> {
    log::debug!(
        "Try to calculate {} hash sum for a given byte buffer. Size: {} bytes",
        algorithm,
        buffer.len()
    );

    get_hash_sum_as_lower_hex(buffer, algorithm)
}

fn get_hash_sum_as_lower_hex<T>(data: T, algorithm: Algorithm) -> Result<String>
where
    T: Chksumable + 'static + Send,
{
    match algorithm {
        Algorithm::MD5 => Ok(calculate_hash_sum::<chksum::MD5, T>(data)?.to_hex_lowercase()),
        Algorithm::SHA1 => Ok(calculate_hash_sum::<chksum::SHA1, T>(data)?.to_hex_lowercase()),
        Algorithm::SHA2_224 => {
            Ok(calculate_hash_sum::<chksum::SHA2_224, T>(data)?.to_hex_lowercase())
        }
        Algorithm::SHA2_256 => {
            Ok(calculate_hash_sum::<chksum::SHA2_256, T>(data)?.to_hex_lowercase())
        }
        Algorithm::SHA2_384 => {
            Ok(calculate_hash_sum::<chksum::SHA2_384, T>(data)?.to_hex_lowercase())
        }
        Algorithm::SHA2_512 => {
            Ok(calculate_hash_sum::<chksum::SHA2_512, T>(data)?.to_hex_lowercase())
        }
    }
}

/// Calculates the hash sum of a given data source, using a specified hashing algorithm.
///
/// # Parameters
/// - `data`: The input data to be hashed. Must implement the [`Chksumable`] trait, enabling it to
///   be processed by the hashing function.
/// - `T`: The type of hashing algorithm to be used, which must implement the `Hash` trait. The
///   output digest of this hash type is expected to be both `Send` and `'static`.
///
/// # Returns
/// - `Result<T::Digest>`: If successful, returns the calculated digest of type `T::Digest`.
///   Otherwise, returns an error if the hash calculation fails.
///
/// # Functionality
/// - Initializes a spinner-style progress bar to indicate the calculation progress to the user.
/// - Spawns a thread to perform the actual hash calculation and send the result back to the main
///   thread through a channel. Upon success, the spinner stops, and the hash digest is returned.
/// - Ensures that all spawned threads are joined (completed) before returning the final result.
///
/// # Errors
/// - If the hash calculation fails, logs the error and returns a descriptive error message.
/// - If there is an issue with sending the hash result back to the main thread, an error will be
///   logged and returned.
///
/// This function is designed for multi-threaded environments where lengthy I/O or CPU-bound operations
/// benefit from non-blocking UI feedback (spinner) and graceful interruption handling.
fn calculate_hash_sum<T, U>(data: U) -> Result<T::Digest>
where
    T: Hash + Send,
    <T as chksum::Hash>::Digest: 'static + Send,
    U: Chksumable + 'static + Send,
{
    // Build a Spinner-Progress-Bar
    let spinner =
        ProgressBar::new_spinner().with_message("Calculate hash sum... this may take a while");

    // Define the spinner style
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&utils::BOUNCING_BAR)
            .template("{spinner:.white} {msg}")
            .unwrap_or(ProgressStyle::default_spinner()),
    );

    // set spinner tick every 100ms
    spinner.enable_steady_tick(Duration::from_millis(100));

    // use thread-safe Channels to transfer the Hash sum to the Main-Thread
    let (sender, receiver) = std::sync::mpsc::channel();

    let hash_sum_thread = thread::spawn(move || -> Result<()> {
        let digest = match chksum::<T>(data) {
            Ok(digest) => digest,
            Err(chksum_err) => {
                spinner.finish_and_clear();
                log::error!(
                    "{}",
                    format!("Failed to calculate hash sum - Details: {:?}", chksum_err)
                );
                return Err(color_eyre::eyre::eyre!(
                    "Failed to calculate hash sum for the specified file."
                ));
            }
        };

        spinner.finish_and_clear();
        sender
            .send(digest)
            .expect("Couldn't send the calculated hash sum via channel to the main thread");
        Ok(())
    });

    // block the main thread until the associated threads are finished
    let hash_sum_result = hash_sum_thread
        .join()
        .expect("Couldn't join on the 'hash sum' thread");

    hash_sum_result?;

    Ok(receiver.try_recv()?)
}

/// Compares the given hashes
pub fn is_hash_equal(origin_hash_sum: &str, calculated_hash_sum: &str) -> bool {
    origin_hash_sum.to_ascii_lowercase() == calculated_hash_sum.to_ascii_lowercase()
}

#[allow(dead_code)]
/// Checks if the given hash is a valid Lower-Hex digit
pub fn is_lower_hex(hash: &str) -> bool {
    !hash.trim().is_empty() && hash.chars().all(|c| matches!(c, 'a'..='f' | '0'..='9'))
}

/// Verifies that every character in the string is a valid hexadecimal digit.
/// Valid hexadecimal (hex) digits are characters that represent numbers in base-16 (hexadecimal) notation.
/// In base-16, digits range from 0 to 15, and these are represented as follows:<br>
/// Decimal 0-9: Represented directly as 0, 1, 2, 3, 4, 5, 6, 7, 8, 9.<br>
/// Decimal 10-15: Represented as letters A, B, C, D, E, F (uppercase) or a, b, c, d, e, f (lowercase).
pub fn is_hash_valid(hash: &str) -> bool {
    !hash.trim().is_empty() && hash.chars().all(|c| c.is_ascii_hexdigit())
}
