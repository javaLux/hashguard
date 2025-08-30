use anyhow::Result;
use clap::{Args, Parser, Subcommand, builder::NonEmptyStringValueParser};
use std::path::PathBuf;

use crate::{
    app::{LogLevel, version},
    filename_handling,
    hasher::{self, Algorithm, HashProperty},
    os_specifics, utils,
};

#[derive(Parser)]
#[command(author, version = version(), about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(
        short,
        long,
        help = "Set up the application logging",
        value_enum,
        value_name = "OPTIONAL"
    )]
    pub logging: Option<LogLevel>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Download a file and calculate a hash sum
    Download(DownloadArgs),
    /// Calculate a hash sum from a file/dir or a byte buffer
    Local(LocalArgs),
}

#[derive(Debug, Args)]
pub struct DownloadArgs {
    #[arg(
        help = "URL to be used for download the file [required]",
        value_name = "URL",
        value_parser = validate_url
    )]
    pub url: String,

    #[arg(
        help = "Original hash sum [optional]",
        value_name = "HASH",
        value_parser = validate_hash
    )]
    pub hash_property: Option<HashProperty>,

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
        help = "Set a directory for the file to be saved (Default is the user's download folder)",
        value_name = "DIR",
        value_parser = validate_output_target
    )]
    pub output: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "Rename the file to be downloaded",
        value_name = "FILE",
        value_parser = validate_file_name
    )]
    pub rename: Option<String>,

    #[arg(
        short,
        long,
        help = "Save the hash to a file, stored in the app data directory"
    )]
    pub save: bool,
}

#[derive(Debug, Args)]
pub struct LocalArgs {
    #[arg(
        help = "Original hash [optional]",
        value_name = "HASH",
        value_parser = validate_hash
    )]
    pub hash_sum: Option<HashProperty>,

    #[arg(
        short,
        long,
        conflicts_with = "buffer",
        help = "Path to a file/dir for which the hash will be calculated",
        value_name = "PATH",
        value_parser = validate_hash_target
    )]
    pub path: Option<PathBuf>,

    #[arg(
        short,
        long,
        conflicts_with = "path",
        help = "Buffer (e.g. String) for which the hash will be calculated",
        value_name = "STRING",
        value_parser = NonEmptyStringValueParser::new()
    )]
    pub buffer: Option<String>,

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
        help = "Include file and directory names in the hash computation [Only has an effect with the option --path]"
    )]
    pub include_names: bool,

    #[arg(
        short,
        long,
        help = "Save the hash to a file, stored in the app data directory"
    )]
    pub save: bool,
}

/// Helper function to validate the option [-o, -output] of the download command
fn validate_output_target(target: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(target);
    if !path.is_dir() {
        let cmd_err = format!(
            "The specified directory '{}' does not exist",
            utils::absolute_path_as_string(&path)
        );
        Err(cmd_err)
    } else {
        Ok(path)
    }
}

/// Helper function to validate the option [-r, -rename] of the download command
fn validate_file_name(filename: &str) -> Result<String, String> {
    // we can use safely `unwrap` because the os type was checked before parsing the cli arguments
    let os_type = os_specifics::get_os().unwrap();
    match filename_handling::validate_filename(&os_type, filename) {
        Ok(_) => Ok(filename.to_string()),
        Err(validate_err) => Err(validate_err.to_string()),
    }
}

/// Helper function to validate option [-p, -path] of the local command
fn validate_hash_target(target: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(target);
    if !path.exists() {
        let cmd_err = format!(
            "The specified path '{}' does not exist",
            utils::absolute_path_as_string(&path)
        );
        Err(cmd_err)
    } else {
        Ok(path)
    }
}

/// Helper function to validate the hash sum argument
fn validate_hash(hash: &str) -> Result<HashProperty, String> {
    match hasher::parse_hash(hash) {
        Ok(hash_property) => Ok(hash_property),
        Err(e) => Err(e.to_string()),
    }
}

/// Helper function to validate the URL argument
fn validate_url(url: &str) -> Result<String, String> {
    if !utils::is_valid_url(url) {
        Err("Failed to parse URL. Please ensure the URL is correctly formatted, including the scheme (e.g. 'http://', 'https://'). For example: https://example.com".to_string())
    } else {
        Ok(url.to_string())
    }
}
