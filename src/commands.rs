use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;

use crate::cli::{DownloadArgs, LocalArgs};
use crate::download::{self, DownloadProperties};
use crate::filename_handling;
use crate::os_specifics;
use crate::utils;
use crate::verify::{self, Algorithm};

#[derive(Debug)]
pub struct CommandResult {
    pub file_location: PathBuf,
    pub used_algorithm: Algorithm,
    pub calculated_hash_sum: String,
    pub hash_compare_result: Option<HashCompareResult>,
}

#[derive(Debug)]
pub struct HashCompareResult {
    pub is_file_modified: bool,
    pub origin_hash_sum: String,
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

pub fn handle_download_cmd(args: DownloadArgs, os_type: os_specifics::OS) -> Result<()> {
    // fetch the output target
    let output_target = args.output;

    let output_target = match output_target {
        Some(output_target) => {
            // only a existing directory is valid as an output target
            let p = Path::new(&output_target);
            if p.is_dir() {
                p.to_path_buf()
            } else {
                let command_err = CommandError::OutputTargetInvalid(utils::get_absolute_path(p));
                return Err(color_eyre::eyre::eyre!(command_err.to_string()));
            }
        }
        // If no output directory was specified
        None => {
            // try to get the default user download folder dependent on the underlying OS
            os_specifics::get_default_download_folder()
        }
    };

    // get the download URL
    let download_url = &args.url;

    if !utils::is_valid_url(download_url) {
        let command_err = CommandError::InvalidUrl;
        return Err(color_eyre::eyre::eyre!(command_err.to_string()));
    }

    let default_file_name = match args.rename {
        Some(file_name) => {
            // validate given file name
            match filename_handling::validate_filename(&os_type, &file_name) {
                Ok(_) => Some(file_name),
                Err(validate_err) => {
                    let command_err = CommandError::RenameOptionInvalid(validate_err.to_string());
                    return Err(color_eyre::eyre::eyre!(command_err.to_string()));
                }
            }
        }
        None => None,
    };

    // build the required DownloadProperties
    let download_properties = DownloadProperties {
        url: download_url.to_string(),
        output_target,
        default_file_name,
        os_type,
    };

    // start the download
    let file_location = download::make_download(download_properties)?;

    let mut calculated_hash_sum =
        verify::get_hash_sum_as_lower_hex(&file_location, args.algorithm)?;

    let cmd_result = if let Some(origin_hash_sum) = args.hash_sum {
        if !verify::is_lower_hex(&origin_hash_sum) {
            // convert the calculated hash sum to UpperHex
            calculated_hash_sum = calculated_hash_sum
                .chars()
                .map(|c| c.to_uppercase().to_string())
                .collect();
        }

        let is_file_modified = verify::is_hash_equal(&origin_hash_sum, &calculated_hash_sum);
        CommandResult {
            file_location,
            used_algorithm: args.algorithm,
            calculated_hash_sum,
            hash_compare_result: Some(HashCompareResult {
                is_file_modified,
                origin_hash_sum,
            }),
        }
    } else {
        CommandResult {
            file_location,
            used_algorithm: args.algorithm,
            calculated_hash_sum,
            hash_compare_result: None,
        }
    };
    utils::processing_cmd_result(cmd_result);
    Ok(())
}

pub fn handle_local_cmd(args: LocalArgs) -> Result<()> {
    let source_file = Path::new(&args.file_path);
    // check if the given file exist
    if source_file.exists() {
        let mut calculated_hash_sum =
            verify::get_hash_sum_as_lower_hex(source_file, args.algorithm)?;

        let cmd_result = if let Some(origin_hash_sum) = args.hash_sum {
            if !verify::is_lower_hex(&origin_hash_sum) {
                // convert the calculated hash sum to UpperHex
                calculated_hash_sum = calculated_hash_sum
                    .chars()
                    .map(|c| c.to_uppercase().to_string())
                    .collect();
            }

            let is_file_modified = verify::is_hash_equal(&origin_hash_sum, &calculated_hash_sum);
            CommandResult {
                file_location: source_file.to_path_buf(),
                used_algorithm: args.algorithm,
                calculated_hash_sum,
                hash_compare_result: Some(HashCompareResult {
                    is_file_modified,
                    origin_hash_sum,
                }),
            }
        } else {
            CommandResult {
                file_location: source_file.to_path_buf(),
                used_algorithm: args.algorithm,
                calculated_hash_sum,
                hash_compare_result: None,
            }
        };
        utils::processing_cmd_result(cmd_result);
        Ok(())
    } else {
        let command_err = CommandError::FileNotExist(utils::get_absolute_path(source_file));
        Err(color_eyre::eyre::eyre!(command_err.to_string()))
    }
}
