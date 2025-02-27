use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::{
    app::{version, LogLevel},
    commands, filename_handling, os_specifics, utils,
    verify::Algorithm,
};

#[derive(Parser)]
#[command(author, version = version(), about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, help = "Set up the application logging", value_enum, value_name = "OPTIONAL")]
    pub logging: Option<LogLevel>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Download a file and calculate a hash sum
    Download(DownloadArgs),
    /// Calculate a hash sum from a file/dir or a byte buffer
    Local(LocalArgs),
}

#[derive(Debug, Args)]
pub struct DownloadArgs {
    /// URL to be used for download the file [required]
    pub url: String,

    /// Origin hash sum [optional]
    pub hash_sum: Option<String>,

    #[arg(
        short,
        long,
        help = "Hash algorithm to be used",
        value_enum,
        default_value_t = Algorithm::default()
    )]
    pub algorithm: Algorithm,

    #[arg(
        short,
        long,
        help = "A custom path for the file to be saved (Default is the user download folder)",
        value_name = "DIR",
        value_parser = validate_output_target
    )]
    pub output: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "Rename the file to be downloaded",
        value_name = "FILE",
        value_parser = check_file_name
    )]
    pub rename: Option<String>,
}

#[derive(Debug, Args)]
pub struct LocalArgs {
    #[arg(
        short,
        long,
        conflicts_with = "buffer",
        help = "Path to a file/dir for which the hash sum will be calculated",
        value_name = "PATH",
        value_parser = validate_hash_target
    )]
    pub path: Option<PathBuf>,

    #[arg(
        short,
        long,
        conflicts_with = "path",
        help = "Buffer (e.g. String) for which the hash sum will be calculated",
        value_name = "STRING"
    )]
    pub buffer: Option<String>,

    /// Origin hash sum [optional]
    pub hash_sum: Option<String>,

    #[arg(
        short,
        long,
        help = "Hash algorithm to be used",
        value_enum,
        default_value_t = Algorithm::default()
    )]
    pub algorithm: Algorithm,
}

/// Helper function to validate the option [-o, -output] of the download command
fn validate_output_target(target: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(target);
    if !path.is_dir() {
        let cmd_err =
            commands::CommandError::OutputTargetInvalid(utils::absolute_path_as_string(&path))
                .to_string();
        Err(cmd_err)
    } else {
        Ok(path)
    }
}

/// Helper function to validate the option [-r, -rename] of the download command
fn check_file_name(filename: &str) -> Result<String, String> {
    // we can use safely `unwrap` because the os type was checked before parsing the cli arguments
    let os_type = os_specifics::get_os().unwrap();
    match filename_handling::validate_filename(&os_type, filename) {
        Ok(_) => Ok(filename.to_string()),
        Err(validate_err) => {
            let cmd_err =
                commands::CommandError::InvalidFilename(validate_err.to_string()).to_string();
            Err(cmd_err)
        }
    }
}

/// Helper function to validate option [-p, -path] of the local command
fn validate_hash_target(target: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(target);
    if !path.exists() {
        let cmd_err =
            commands::CommandError::PathNotExist(utils::absolute_path_as_string(&path)).to_string();
        Err(cmd_err)
    } else {
        Ok(path)
    }
}
