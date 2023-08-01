use chksum::{
    prelude::{self, HashDigest},
    Chksum,
};
use clap::ValueEnum;
use indicatif::{ProgressBar, ProgressStyle};
use std::{error::Error, fs::File, path::PathBuf, sync::mpsc, thread, time::Duration};

use crate::color_templates::ERROR_TEMPLATE_NO_BG_COLOR;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
/// Indicate the supported Hash-Algorithm to build the file hash sum
pub enum Algorithm {
    MD5,
    SHA1,
    SHA2_256,
    SHA2_512,
}

fn generate_hash_sum(mut file: File, hash_alg: prelude::HashAlgorithm) -> HashDigest {
    // Build a Spinner-Progress-Bar
    let spinner = ProgressBar::new_spinner().with_message("Generate hash sum...");

    // Define the spinner style
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("|/--\\")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    // set spinner tick every 100ms
    spinner.enable_steady_tick(Duration::from_millis(100));

    // use thread-safe Channels to transfer the Hash sum to the Main-Thread
    let (sender, receiver) = mpsc::channel();

    let handle_thread = thread::spawn(move || {
        let result = file.chksum(hash_alg);

        if let Err(err) = result {
            println!(
                "{}: {}",
                ERROR_TEMPLATE_NO_BG_COLOR.output("Failed to generate Hash sum"),
                err
            );
            std::process::exit(1);
        } else {
            spinner.finish_and_clear();
            // send result of calculating the Hash sum to the Main-Thread
            sender.send(result.ok().unwrap()).unwrap();
        }
    });

    // block the main thread until the associated thread is finished
    handle_thread.join().unwrap();

    let receive_result = receiver.recv();
    if let Err(err) = receive_result {
        println!(
            "{}: {}",
            ERROR_TEMPLATE_NO_BG_COLOR.output("Failed to generate Hash sum"),
            err
        );
        std::process::exit(1);
    } else {
        // return the calculate file hash sum
        receive_result.ok().unwrap()
    }
}

pub fn is_file_modified(
    path: &PathBuf,
    origin_hash_sum: &str,
    algorithm: Option<Algorithm>,
) -> Result<bool, Box<dyn Error>> {
    let mut is_modified = false;

    // create file object
    let file_to_check = File::open(path)?;

    // build the file hash sum dependent on the given algorithm
    if algorithm.is_some() {
        match algorithm {
            Some(Algorithm::MD5) => {
                let generated_hash_sum =
                    generate_hash_sum(file_to_check, prelude::HashAlgorithm::MD5);

                is_modified = compare_hash_sums(origin_hash_sum, generated_hash_sum);
            }
            Some(Algorithm::SHA1) => {
                let generated_hash_sum =
                    generate_hash_sum(file_to_check, prelude::HashAlgorithm::SHA1);

                is_modified = compare_hash_sums(origin_hash_sum, generated_hash_sum);
            }
            Some(Algorithm::SHA2_256) => {
                let generated_hash_sum =
                    generate_hash_sum(file_to_check, prelude::HashAlgorithm::SHA2_256);

                is_modified = compare_hash_sums(origin_hash_sum, generated_hash_sum);
            }
            Some(Algorithm::SHA2_512) => {
                let generated_hash_sum =
                    generate_hash_sum(file_to_check, prelude::HashAlgorithm::SHA2_512);

                is_modified = compare_hash_sums(origin_hash_sum, generated_hash_sum);
            }
            _ => {}
        }
    } else {
        // if no specific algorithm given use SHA2_256 as default
        let generated_hash_sum = generate_hash_sum(file_to_check, prelude::HashAlgorithm::SHA2_256);

        is_modified = compare_hash_sums(origin_hash_sum, generated_hash_sum);
    }

    Ok(is_modified)
}

/// Compare the origin hash sum with the generated hash sum
fn compare_hash_sums(origin_hash_sum: &str, generated_hash_sum: HashDigest) -> bool {
    // check if the origin hash sum is a Upper-Hex or Lower-Hex number
    let generated_hash_sum = if is_lower_hex(origin_hash_sum) {
        // convert the generated hash sum to a Lower-Hex number
        format!("{:x}", generated_hash_sum)
    } else {
        // or to a Upper-Hex number
        format!("{:X}", generated_hash_sum)
    };

    println!("Origin hash sum   : {}", origin_hash_sum);
    println!("Generated hash sum: {}", generated_hash_sum);
    // compare hash sums
    origin_hash_sum != generated_hash_sum
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
