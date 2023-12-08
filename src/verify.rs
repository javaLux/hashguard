use chksum::{
    chksum,
    hash::{Digest, MD5, SHA1, SHA2_256, SHA2_512},
    Chksum, Error as ChksumError,
};
use clap::ValueEnum;
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs::File, path::PathBuf, sync::mpsc, thread, time::Duration};

use color_eyre::Result;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
/// Indicate the supported Hash-Algorithm to build the file hash sum
pub enum Algorithm {
    MD5,
    SHA1,
    SHA2_256,
    SHA2_512,
}

/// Calculate a hash sum dependent on the given hash sum algorithm
fn calculate_hash_sum<T: Chksum<File, Error = ChksumError>>(file: File) -> Result<T::Digest>
where
    T::Digest: 'static + Send,
{
    // Build a Spinner-Progress-Bar
    let spinner = ProgressBar::new_spinner().with_message("Generate hash sum...");

    // Define the spinner style
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("|/--\\")
            .template("{spinner:.green} {msg}")?,
    );

    // set spinner tick every 100ms
    spinner.enable_steady_tick(Duration::from_millis(100));

    // use thread-safe Channels to transfer the Hash sum to the Main-Thread
    let (sender, receiver) = mpsc::channel();

    let hash_sum_thread = thread::spawn(move || -> Result<()> {
        let digest = chksum::<T, _>(file)?;

        spinner.finish_and_clear();
        sender
            .send(digest)
            .expect("Couldn't send calculated hash sum over channel");
        Ok(())
    });

    // block the main thread until the associated thread is finished
    let hash_sum_result = hash_sum_thread
        .join()
        .expect("Couldn't join on the hash sum thread");

    hash_sum_result?;

    Ok(receiver.try_recv()?)
}

pub fn is_file_modified(
    path: &PathBuf,
    origin_hash_sum: &str,
    algorithm: Option<Algorithm>,
) -> Result<bool> {
    // create file object
    let file_to_check = File::open(path)?;

    match algorithm {
        Some(algorithm) => match algorithm {
            Algorithm::MD5 => {
                let calculated_hash_sum = calculate_hash_sum::<MD5>(file_to_check)?;

                Ok(is_hash_sum_equal(origin_hash_sum, calculated_hash_sum))
            }
            Algorithm::SHA1 => {
                let calculated_hash_sum = calculate_hash_sum::<SHA1>(file_to_check)?;

                Ok(is_hash_sum_equal(origin_hash_sum, calculated_hash_sum))
            }
            Algorithm::SHA2_256 => {
                let calculated_hash_sum = calculate_hash_sum::<SHA2_256>(file_to_check)?;

                Ok(is_hash_sum_equal(origin_hash_sum, calculated_hash_sum))
            }
            Algorithm::SHA2_512 => {
                let calculated_hash_sum = calculate_hash_sum::<SHA2_512>(file_to_check)?;

                Ok(is_hash_sum_equal(origin_hash_sum, calculated_hash_sum))
            }
        },
        None => {
            // if no specific algorithm given use SHA2_256 as default
            let calculated_hash_sum = calculate_hash_sum::<SHA2_256>(file_to_check)?;

            Ok(is_hash_sum_equal(origin_hash_sum, calculated_hash_sum))
        }
    }
}

/// Compare the origin hash sum with the calculated hash sum
fn is_hash_sum_equal<T: Digest>(origin_hash_sum: &str, calculated_hash_sum: T) -> bool {
    // check if the origin hash sum is a Upper-Hex or Lower-Hex number
    let calculated_hash_sum = if is_lower_hex(origin_hash_sum) {
        // convert the generated hash sum to a Lower-Hex number
        format!("{:x}", calculated_hash_sum)
    } else {
        // or to a Upper-Hex number
        format!("{:X}", calculated_hash_sum)
    };

    println!("Origin hash sum    : {}", origin_hash_sum);
    println!("Calculated hash sum: {}", calculated_hash_sum);
    // compare hash sums
    origin_hash_sum != calculated_hash_sum
}

#[allow(dead_code)]
/// Checks if the given hash sum is a Upper-Hex number
fn is_upper_hex(chk_sum: &str) -> bool {
    for char in chk_sum.chars() {
        if char.is_ascii_uppercase() {
            return true;
        }
    }

    false
}

/// Checks if the given hash sum is a Lower-Hex number
fn is_lower_hex(chk_sum: &str) -> bool {
    for char in chk_sum.chars() {
        if char.is_ascii_lowercase() {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lower_hex() {
        let check_sum = "c92fae5e42b9aecf444a66c8ec563c652f60b1e231dfdd33e";
        assert!(is_lower_hex(check_sum));
    }

    #[test]
    fn test_upper_hex() {
        let check_sum = "A92fAE5G42B9F444";
        assert!(is_upper_hex(check_sum));
    }
}
