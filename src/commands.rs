use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use color_eyre::Result;

use crate::cli::{DownloadArgs, LocalArgs};
use crate::color_templates::WARN_TEMPLATE_NO_BG_COLOR;
use crate::download;
use crate::filename_handling;
use crate::os_specifics;
use crate::util;
use crate::verify;

#[derive(Debug)]
pub struct DownloadCommandResult {
    pub is_file_modified: bool,
    pub file_destination: PathBuf,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum CommandError {
    FileNotExist(String),
    InvalidUrl,
    OutputTargetInvalid(String),
    RenameOptionInvalid(String),
}

impl Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::FileNotExist(file) => {
                let msg = format!("The specified file '{}' does not exist", file);
                write!(f, "{}", msg)
            }
            CommandError::InvalidUrl => {
                write!(f, "The specified url is invalid")
            }
            CommandError::OutputTargetInvalid(target) => {
                let msg = format!(
                    "The specified output target '{}' does not exist or is not a directory",
                    target
                );
                write!(f, "{}", msg)
            }
            CommandError::RenameOptionInvalid(filename_err) => {
                let msg = format!(
                    "Option [-r, --rename] contains an invalid filename - {}",
                    filename_err
                );
                write!(f, "{}", msg)
            }
        }
    }
}

/// Download a file from the internet, calculate a hash sum for an given local file and compare the generated hash sum with the origin hash sum
pub fn perform_download_command(
    download_args: DownloadArgs,
    os_type: os_specifics::OS,
) -> Result<DownloadCommandResult> {
    // fetch the output target
    let output_target = download_args.output;

    let output_target = match output_target {
        Some(output_target) => {
            // only a existing directory is valid as an output target
            if Path::new(&output_target).is_dir() {
                output_target
            } else {
                let command_err = CommandError::OutputTargetInvalid(output_target);
                return Err(command_err.into());
            }
        }
        // If no output directory was specified
        None => {
            // try to get the default user download folder dependent on the underlying OS
            let download_folder = os_specifics::get_default_download_folder(&os_type);

            match download_folder {
                // The user's download folder was successfully determined
                Some(download_folder) => download_folder,
                None => {
                    // Otherwise, use the current directory as the storage location
                    let current_working_dir = std::env::current_dir()?;
                    current_working_dir.to_string_lossy().to_string()
                }
            }
        }
    };

    // get the download URL
    let download_url = &download_args.url;

    if util::is_valid_url(download_url) {
        // dependent on the given cli options, control the filename
        let file_name: Result<String> = match download_args.rename {
            Some(filename) => match filename_handling::validate_filename(&os_type, &filename) {
                Ok(_) => Ok(filename.to_string()),
                Err(filename_err) => {
                    let command_err = CommandError::RenameOptionInvalid(filename_err.to_string());
                    Err(command_err.into())
                }
            },
            None => {
                // no rename option given, try to extract file name from url
                match util::extract_file_name_from_url(download_url) {
                    Some(filename) => {
                        // check if this is a valid filename depending on the rules of the underlying OS
                        match filename_handling::validate_filename(&os_type, filename) {
                            Ok(_) => Ok(filename.to_string()),
                            Err(_) => {
                                println!(
                                    "{}",
                                    WARN_TEMPLATE_NO_BG_COLOR
                                        .output("The extracted filename from the URL is invalid")
                                );

                                println!("Please enter a name for the file to be downloaded");
                                // Make Input-Prompt and return the valid filename
                                Ok(filename_handling::enter_and_verify_file_name(&os_type)?)
                            }
                        }
                    }
                    None => {
                        // no filename was found in the given URL -> enter a custom filename
                        println!(
                            "{}",
                            WARN_TEMPLATE_NO_BG_COLOR.output("Could not extract filename from URL")
                        );

                        println!("Please enter a name for the file to be downloaded");
                        // Make Input-Prompt and return the valid filename
                        Ok(filename_handling::enter_and_verify_file_name(&os_type)?)
                    }
                }
            }
        };

        // check the filename result
        let file_name = file_name?;

        // build the final destination path for the file to be download
        let mut file_destination = PathBuf::new();
        file_destination.push(output_target);
        file_destination.push(file_name);

        // start the file download
        download::download_file(download_url, &file_destination)?;

        // build the hash sum from the downloaded file and compare with the origin hash sum
        let is_file_modified = verify::is_file_modified(
            &file_destination,
            &download_args.hash_sum,
            download_args.algorithm,
        )?;

        // Finally build the result for the download command and return them
        Ok(DownloadCommandResult {
            is_file_modified,
            file_destination,
        })
    } else {
        let command_err = CommandError::InvalidUrl;
        Err(command_err.into())
    }
}

/// Calculate a hash sum for an given local file and compare the generated hash sum with the origin hash sum
pub fn is_local_file_modified(local_args: LocalArgs) -> Result<bool> {
    // create Path-Object from given file path
    let source_file = Path::new(&local_args.file_path);

    // check if the given file exist
    if source_file.exists() {
        // calculate a hash sum from given file and compare with the origin hash sum
        let is_file_modified = verify::is_file_modified(
            &source_file.to_path_buf(),
            &local_args.hash_sum,
            local_args.algorithm,
        )?;
        Ok(is_file_modified)
    } else {
        let source_file_path = source_file
            .as_os_str()
            .to_owned()
            .to_string_lossy()
            .to_string();
        let command_err = CommandError::FileNotExist(source_file_path);
        Err(command_err.into())
    }
}
