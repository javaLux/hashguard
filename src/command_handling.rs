use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use anyhow::Result;

use crate::{
    cli::{DownloadArgs, LocalArgs},
    download::{self, DownloadProperties},
    hasher::{self, Algorithm},
    local, os_specifics, utils,
};

#[derive(Debug)]
pub struct CommandResult {
    pub file_location: Option<PathBuf>,
    pub buffer: Option<String>,
    pub used_algorithm: Algorithm,
    pub calculated_hash_sum: String,
    pub hash_compare_result: Option<HashCompareResult>,
    pub save: bool,
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
        // let prefix = "Validation failed →";
        match self {
            CommandValidationError::PathNotExist(path) => {
                let msg = format!("The specified path '{path}' does not exist");
                write!(f, "{msg}")
            }
            CommandValidationError::InvalidUrl => {
                write!(
                    f,
                    "The specified URL is invalid. Please ensure the URL is correctly formatted, including the scheme (e.g. 'http://', 'https://'). For example: https://example.com"
                )
            }
            CommandValidationError::InvalidHashSum => {
                write!(f, "The specified hash sum is not a valid hexadecimal digit")
            }
            CommandValidationError::OutputTargetInvalid(target) => {
                let msg = format!(
                    "Invalid output target - '{target}' does not exist or is not a directory"
                );
                write!(f, "{msg}")
            }
            CommandValidationError::InvalidFilename(filename_err) => {
                let msg = format!("Invalid filename - {filename_err}");
                write!(f, "{msg}")
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

    // build the required DownloadProperties
    let download_properties = DownloadProperties {
        algorithm: args.algorithm,
        url: download_url.to_string(),
        output_target,
        default_file_name: args.rename,
        os_type,
    };

    // start the download
    let download_result = download::execute_download(download_properties)?;

    let cmd_result = if let Some(origin_hash_sum) = args.hash_sum {
        let is_hash_equal = hasher::is_hash_equal(&origin_hash_sum, &download_result.hash_sum);

        CommandResult {
            file_location: Some(download_result.file_location),
            buffer: None,
            used_algorithm: args.algorithm,
            calculated_hash_sum: download_result.hash_sum,
            hash_compare_result: Some(HashCompareResult {
                is_hash_equal,
                origin_hash_sum,
            }),
            save: args.save,
        }
    } else {
        CommandResult {
            file_location: Some(download_result.file_location),
            buffer: None,
            used_algorithm: args.algorithm,
            calculated_hash_sum: download_result.hash_sum,
            hash_compare_result: None,
            save: args.save,
        }
    };
    utils::processing_cmd_result(&cmd_result)?;

    Ok(())
}

// Handle the CLI subcommand 'local'
pub fn handle_local_cmd(args: LocalArgs) -> Result<()> {
    let (calculated_hash_sum, file_location, buffer) = if let Some(path) = args.path {
        // calculate the file hash
        let calculated_hash_sum =
            local::get_hash_for_object(path.clone(), args.algorithm, args.include_names)?;
        (calculated_hash_sum, Some(path), None)
    } else if let Some(some_text) = args.buffer {
        let buffer = some_text.as_bytes().to_vec();
        let calculated_hash_sum = local::get_buffer_hash(&buffer, args.algorithm);
        (calculated_hash_sum, None, Some(some_text))
    } else {
        return Err(anyhow::anyhow!(
            "Either a path or a buffer must be provided."
        ));
    };

    let cmd_result = if let Some(origin_hash_sum) = args.hash_sum {
        let is_hash_equal = hasher::is_hash_equal(&origin_hash_sum, &calculated_hash_sum);

        CommandResult {
            file_location,
            buffer,
            used_algorithm: args.algorithm,
            calculated_hash_sum: calculated_hash_sum.to_string(),
            hash_compare_result: Some(HashCompareResult {
                is_hash_equal,
                origin_hash_sum,
            }),
            save: args.save,
        }
    } else {
        CommandResult {
            file_location,
            buffer,
            used_algorithm: args.algorithm,
            calculated_hash_sum: calculated_hash_sum.to_string(),
            hash_compare_result: None,
            save: args.save,
        }
    };
    utils::processing_cmd_result(&cmd_result)?;

    Ok(())
}
