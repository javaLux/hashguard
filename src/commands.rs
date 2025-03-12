use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use anyhow::Result;

use crate::cli::{DownloadArgs, LocalArgs};
use crate::download::{self, DownloadProperties};
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
    pub is_hash_equal: bool,
    pub origin_hash_sum: String,
}

// Represents possible cli command errors
#[derive(Debug, PartialEq, PartialOrd)]
pub enum CommandValidationError {
    PathNotExist(String),
    InvalidUrl,
    InvalidHashSum,
    OutputTargetInvalid(String),
    InvalidFilename(String),
}

impl Error for CommandValidationError {}

impl fmt::Display for CommandValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = "Validation failed →";
        match self {
            CommandValidationError::PathNotExist(path) => {
                let msg = format!("The specified path '{}' does not exist", path);
                write!(f, "{}", msg)
            }
            CommandValidationError::InvalidUrl => {
                write!(f, "{prefix} The specified URL is invalid. Please ensure the URL is correctly formatted, including the scheme (e.g. 'http://', 'https://'). For example: https://example.com")
            }
            CommandValidationError::InvalidHashSum => {
                write!(
                    f,
                    "{prefix} The specified hash sum is not a valid hexadecimal digit"
                )
            }
            CommandValidationError::OutputTargetInvalid(target) => {
                let msg = format!(
                    "Invalid output target - '{}' does not exist or is not a directory",
                    target
                );
                write!(f, "{}", msg)
            }
            CommandValidationError::InvalidFilename(filename_err) => {
                let msg = format!("Invalid filename - {}", filename_err);
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
        Some(output_target) => output_target,
        // If no output directory was specified
        None => {
            // try to get the default user download folder dependent on the underlying OS
            os_specifics::download_directory()
        }
    };

    // get the download URL
    let download_url = &args.url;

    if !utils::is_valid_url(download_url) {
        let command_err = CommandValidationError::InvalidUrl;
        return Err(command_err.into());
    }

    // Check if the provided hash is a valid hex digit
    if let Some(origin_hash_sum) = args.hash_sum.as_ref() {
        if !verify::is_hash_valid(origin_hash_sum) {
            return Err(CommandValidationError::InvalidHashSum.into());
        }
    }

    // build the required DownloadProperties
    let download_properties = DownloadProperties {
        url: download_url.to_string(),
        output_target,
        default_file_name: args.rename,
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
    let (calculated_hash_sum, file_location, buffer_size) = if let Some(path) = args.path {
        // calculate the file hash
        let calculated_hash_sum = verify::get_file_hash(path.clone(), args.algorithm)?;
        (calculated_hash_sum, Some(path), None)
    } else if let Some(some_text) = args.buffer {
        let buffer = some_text.as_bytes().to_vec();
        let buffer_size = buffer.len();
        let calculated_hash_sum = verify::get_buffer_hash(buffer, args.algorithm)?;
        (calculated_hash_sum, None, Some(buffer_size))
    } else {
        return Ok(());
    };

    let cmd_result = if let Some(origin_hash_sum) = args.hash_sum {
        // Check if the provided hash is a valid hex digit
        if !verify::is_hash_valid(&origin_hash_sum) {
            return Err(CommandValidationError::InvalidHashSum.into());
        }

        let is_hash_equal = verify::is_hash_equal(&origin_hash_sum, &calculated_hash_sum);
        CommandResult {
            file_location,
            buffer_size,
            used_algorithm: args.algorithm,
            calculated_hash_sum: calculated_hash_sum.to_string(),
            hash_compare_result: Some(HashCompareResult {
                is_hash_equal,
                origin_hash_sum,
            }),
        }
    } else {
        CommandResult {
            file_location,
            buffer_size,
            used_algorithm: args.algorithm,
            calculated_hash_sum: calculated_hash_sum.to_string(),
            hash_compare_result: None,
        }
    };
    utils::processing_cmd_result(cmd_result);

    Ok(())
}
