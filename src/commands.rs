use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use color_eyre::eyre::Result;

use crate::cli::{DownloadArgs, LocalArgs};
use crate::download::{self, DownloadProperties};
use crate::filename_handling;
use crate::os_specifics;
use crate::utils;
use crate::verify::{self, Algorithm};

#[derive(Debug)]
pub struct CommandResult {
    pub file_location: Option<PathBuf>,
    pub buffer_size: Option<usize>,
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
    PathNotExist(String),
    InvalidUrl,
    OutputTargetInvalid(String),
    RenameOptionInvalid(String),
}

impl Error for CommandError {}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::PathNotExist(path) => {
                let msg = format!("The specified path '{}' does not exist", path);
                write!(f, "{}", msg)
            }
            CommandError::InvalidUrl => {
                write!(f, "The provided URL is invalid. Please ensure the URL is correctly formatted,\nincluding the scheme (e.g. 'http://', 'https://').\nFor example: https://example.com")
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

// Handle the CLI subcommand 'download'
pub fn handle_download_cmd(args: DownloadArgs, os_type: os_specifics::OS) -> Result<()> {
    // fetch the output target
    let output_target = args.output;

    let output_target = match output_target {
        Some(output_target) => {
            // only a existing directory is valid as an output target
            if output_target.is_dir() {
                output_target
            } else {
                let command_err =
                    CommandError::OutputTargetInvalid(utils::get_absolute_path(&output_target));
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
    let file_path = download::execute_download(download_properties)?;

    let local_args = LocalArgs {
        path: Some(file_path),
        buffer: None,
        hash_sum: args.hash_sum,
        algorithm: args.algorithm,
    };

    handle_local_cmd(local_args)
}

// Handle the CLI subcommand 'local'
pub fn handle_local_cmd(args: LocalArgs) -> Result<()> {
    let (mut calculated_hash_sum, file_location, buffer_size) = if let Some(path) = args.path {
        // check if the given path point to an existing file or directory
        if path.exists() {
            let calculated_hash_sum = verify::get_file_hash(path.clone(), args.algorithm)?;
            (calculated_hash_sum, Some(path), None)
        } else {
            let command_err = CommandError::PathNotExist(utils::get_absolute_path(&path));
            return Err(color_eyre::eyre::eyre!(command_err.to_string()));
        }
    } else if let Some(some_text) = args.buffer {
        let buffer = some_text.as_bytes().to_vec();
        let buffer_size = buffer.len();
        let calculated_hash_sum = verify::get_buffer_hash(buffer, args.algorithm)?;
        (calculated_hash_sum, None, Some(buffer_size))
    } else {
        return Ok(());
    };

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
            buffer_size,
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
            buffer_size,
            used_algorithm: args.algorithm,
            calculated_hash_sum,
            hash_compare_result: None,
        }
    };
    utils::processing_cmd_result(cmd_result);

    Ok(())
}
