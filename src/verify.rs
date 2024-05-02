#![allow(where_clauses_object_safety)]
use chksum::{chksum, Hash};
use clap::ValueEnum;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs::File,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use color_eyre::eyre::Result;

use crate::{app, utils};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
/// Indicate the supported Hash-Algorithm to build the file hash sum
pub enum Algorithm {
    MD5,
    SHA1,
    #[default]
    SHA2_256,
    SHA2_512,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Algorithm::MD5 => write!(f, "MD5"),
            Algorithm::SHA1 => write!(f, "SHA1"),
            Algorithm::SHA2_256 => write!(f, "SHA2-256"),
            Algorithm::SHA2_512 => write!(f, "SHA2-512"),
        }
    }
}

/// Calculates the hash sum for a given file using the specified hash algorithm and returns
/// the result as a lowercase hexadecimal string.
///
/// # Arguments
///
/// * `path`: The path to the file for which the hash sum needs to be calculated.
/// * `algorithm`: The hash algorithm to use (MD5, SHA1, SHA2_256, or SHA2_512).
///
/// # Returns
///
/// Returns a `Result<String>` containing the lowercase hexadecimal representation of
/// the calculated hash sum if the operation is successful. Otherwise, returns an error.
pub fn get_hash_sum_as_lower_hex(path: &Path, algorithm: Algorithm) -> Result<String> {
    log::debug!(
        "Try to calculate {} hash sum for file: {}",
        algorithm,
        utils::get_absolute_path(path)
    );
    // create file object
    let file_to_check = File::open(path)?;

    match algorithm {
        Algorithm::MD5 => Ok(calculate_hash_sum::<chksum::MD5>(file_to_check)?.to_hex_lowercase()),
        Algorithm::SHA1 => {
            Ok(calculate_hash_sum::<chksum::SHA1>(file_to_check)?.to_hex_lowercase())
        }
        Algorithm::SHA2_256 => {
            Ok(calculate_hash_sum::<chksum::SHA2_256>(file_to_check)?.to_hex_lowercase())
        }
        Algorithm::SHA2_512 => {
            Ok(calculate_hash_sum::<chksum::SHA2_512>(file_to_check)?.to_hex_lowercase())
        }
    }
}

/// Calculate a hash sum dependent on the given hash sum algorithm
fn calculate_hash_sum<T>(data: File) -> Result<T::Digest>
where
    T: Hash + Send,
    <T as chksum::Hash>::Digest: 'static + Send,
{
    // Build a Spinner-Progress-Bar
    let spinner =
        ProgressBar::new_spinner().with_message("Calculate hash sum... this may take a while");

    // Define the spinner style
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&utils::BOUNCING_BAR)
            .template("{spinner:.white} {msg}")?,
    );

    // set spinner tick every 100ms
    spinner.enable_steady_tick(Duration::from_millis(100));

    let is_calculating_checksum = Arc::new(AtomicBool::new(true));

    let is_calculating_checksum_clone = is_calculating_checksum.clone();

    // thread to listen on the app state -> if ctrl_c was pressed, quit the app
    let listen_crl_c_thread = thread::spawn(move || {
        while is_calculating_checksum.load(Ordering::SeqCst) {
            // listen to the app state
            if !app::APP_SHOULD_RUN.load(Ordering::SeqCst) {
                log::debug!("{} was interrupted by user...", app::APP_NAME);
                println!("{}", app::APP_INTERRUPTED_MSG);
                // terminate app
                std::process::exit(1);
            }
        }
    });

    // use thread-safe Channels to transfer the Hash sum to the Main-Thread
    let (sender, receiver) = mpsc::channel();

    let hash_sum_thread = thread::spawn(move || -> Result<()> {
        let digest = match chksum::<T>(data) {
            Ok(digest) => {
                is_calculating_checksum_clone.store(false, Ordering::SeqCst);
                digest
            }
            Err(chksum_err) => {
                is_calculating_checksum_clone.store(false, Ordering::SeqCst);
                spinner.finish_and_clear();
                return Err(color_eyre::eyre::eyre!(chksum_err));
            }
        };

        spinner.finish_and_clear();
        sender
            .send(digest)
            .expect("Couldn't send the calculated hash sum via channel to the main thread");
        Ok(())
    });

    // block the main thread until the associated threads are finished
    listen_crl_c_thread
        .join()
        .expect("Couldn't join on the 'listen_ctrl_c' thread");

    let hash_sum_result = hash_sum_thread
        .join()
        .expect("Couldn't join on the 'hash sum' thread");

    hash_sum_result?;

    Ok(receiver.try_recv()?)
}

pub fn is_hash_equal(origin_hash_sum: &str, calculated_hash_sum: &str) -> bool {
    // compare hash sums
    origin_hash_sum != calculated_hash_sum
}

#[allow(dead_code)]
/// Checks if the given hash sum is a Upper-Hex number
pub fn is_upper_hex(chk_sum: &str) -> bool {
    for char in chk_sum.chars() {
        if char.is_ascii_uppercase() {
            return true;
        }
    }

    false
}

/// Checks if the given hash sum is a Lower-Hex number
pub fn is_lower_hex(chk_sum: &str) -> bool {
    for char in chk_sum.chars() {
        if char.is_ascii_lowercase() {
            return true;
        }
    }

    false
}
