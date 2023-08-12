use clap::Parser;
use std::path::{Path, PathBuf};

// make own custom modules available
pub mod cli;
pub mod color_templates;
pub mod download;
pub mod filename_handling;
pub mod os_specifics;
pub mod util;
pub mod verify;

use cli::{DownloadArgs, LocalArgs};
use color_templates::*;
use os_specifics::OS;

fn main() {
    // Parse the given CLI-Arguments
    let cli_args = cli::Cli::parse();

    // After parsing the cli args, check if the underlying OS is supported
    let os_type = os_specifics::get_os();

    // Only Linux, MacOsX and Windows are supported
    if os_type.is_none() {
        println!(
            "{} - Supported OS: {}",
            WARN_TEMPLATE_NO_BG_COLOR
                .output("Unable to execute, your current Operating-System is unsupported."),
            INFO_TEMPLATE.output(format!(
                "[{:?}, {:?}, {:?}]",
                os_specifics::OS::Linux,
                os_specifics::OS::MacOsX,
                os_specifics::OS::Windows
            )),
        );

        std::process::exit(0);
    }

    // take the given OS-Type
    let os_type = os_type.unwrap();

    // check which command is given (download or local)
    match cli_args.command {
        cli::Commands::Download(download_args) => perform_download_command(download_args, os_type),
        cli::Commands::Local(local_args) => perform_local_command(local_args),
    }
}

fn perform_download_command(download_args: DownloadArgs, os_type: OS) {
    // check and verify a given output location
    let output_dir = download_args.output;

    let output_dir = match output_dir {
        Some(dir) => {
            // only a existing directory is valid as output directory
            if Path::new(&dir).is_dir() {
                dir
            } else {
                println!(
                    "{} > {} < {}",
                    ERROR_TEMPLATE_NO_BG_COLOR.output("The specified output target"),
                    dir,
                    ERROR_TEMPLATE_NO_BG_COLOR.output("does not exist or is not a directory")
                );
                std::process::exit(0);
            }
        }
        None => {
            // try to get the default user download folder dependent on the underlying OS
            let download_folder = os_specifics::get_default_download_folder(&os_type);

            if let Some(download_folder) = download_folder {
                download_folder
            } else {
                // take the current working directory as file location
                std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            }
        }
    };

    // get the download URL
    let download_url = &download_args.url;

    // check if the given download URL is a valid URL
    if util::is_valid_url(download_url) {
        // extract the filename (last part of the URL)
        let extract_result = util::extract_file_name_from_url(download_url);

        // check filename if present and if valid
        let file_name = match extract_result {
            Some(file_name) => {
                // check if this is a valid filename depending on the rules of the underlying OS
                if filename_handling::is_valid_filename(&os_type, file_name) {
                    file_name.to_string()
                } else {
                    println!(
                        "{} -> Specify your own filename",
                        WARN_TEMPLATE_NO_BG_COLOR
                            .output("The extracted filename from the URL is invalid")
                    );
                    // Make Input-Prompt and return the valid filename
                    filename_handling::enter_and_verify_file_name(&os_type)
                }
            }
            None => {
                // no filename was found in the given URL -> enter a custom filename
                println!(
                    "{} -> Specify your own filename",
                    WARN_TEMPLATE_NO_BG_COLOR.output("Could not extract filename from URL")
                );

                // Make Input-Prompt and return filename
                filename_handling::enter_and_verify_file_name(&os_type)
            }
        };

        // build the final destination path for the file to be download
        let mut final_file_path = PathBuf::new();
        final_file_path.push(output_dir);
        final_file_path.push(file_name);

        // start the file download
        let result = download::download_file(download_url, &final_file_path);

        if result.is_ok() {
            // build the hash sum from the downloaded file and compare with the origin hash sum
            let is_modified = verify::is_file_modified(
                &final_file_path,
                &download_args.hash_sum,
                download_args.algorithm,
            );

            match is_modified {
                Ok(false) => {
                    println!("{}", INFO_TEMPLATE.output("Hash sum match"));
                }
                Ok(true) => {
                    println!("{}", ERROR_TEMPLATE.output("Hash sum do not match"));
                }
                Err(error) => {
                    println!(
                        "{}: {}",
                        ERROR_TEMPLATE_NO_BG_COLOR.output("File could not be processed"),
                        error
                    );
                }
            }

            // Finally print the file location
            println!(
                "File location: {}",
                WARN_TEMPLATE_NO_BG_COLOR.output(final_file_path.display())
            );
        } else {
            println!(
                "{}",
                ERROR_TEMPLATE_NO_BG_COLOR.output(result.err().unwrap())
            );
        }
    } else {
        println!(
            "{} - Specify a valid URL for the file download",
            color_templates::WARN_TEMPLATE_NO_BG_COLOR.output("Invalid URL")
        );
    }
}

fn perform_local_command(local_args: LocalArgs) {
    // create Path-Object from given file path
    let source_file = Path::new(&local_args.file_path);

    // check if given file exists
    if source_file.exists() {
        // build hash sum from given file and compare with the given origin hash sum
        let is_modified = verify::is_file_modified(
            &source_file.to_path_buf(),
            &local_args.hash_sum,
            local_args.algorithm,
        );

        match is_modified {
            Ok(false) => {
                println!("{}", INFO_TEMPLATE.output("Hash sum match"));
            }
            Ok(true) => {
                println!("{}", ERROR_TEMPLATE.output("Hash sum do not match"));
            }
            Err(error) => {
                println!(
                    "{}: {}",
                    ERROR_TEMPLATE_NO_BG_COLOR.output("File could not be processed"),
                    error
                );
            }
        }
    } else {
        println!(
            "{} - Specify a path to an existing file",
            color_templates::WARN_TEMPLATE_NO_BG_COLOR.output("Given file does no exist")
        );
    }
}
