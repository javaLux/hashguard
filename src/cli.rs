use clap::{Args, Parser, Subcommand};

use crate::{
    app::{version, LogLevel},
    verify::Algorithm,
};

#[derive(Parser)]
#[command(author, version = version(), about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        short,
        long,
        help = "Use debug to display backtrace and to write a log file",
        value_enum
    )]
    pub log_level: Option<LogLevel>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Download a file and calculate a hash sum or check with a specific hash sum
    Download(DownloadArgs),
    /// Calculate a hash sum from a local file or check with a specific hash sum
    Local(LocalArgs),
}

#[derive(Debug, Args)]
pub struct DownloadArgs {
    /// URL to be used for download the file [required]
    pub url: String,

    /// Origin hash sum of the file [optional]
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
        value_name = "DIR"
    )]
    pub output: Option<String>,

    #[arg(
        short,
        long,
        help = "Rename the file to be downloaded",
        value_name = "FILE"
    )]
    pub rename: Option<String>,
}

#[derive(Debug, Args)]
pub struct LocalArgs {
    /// Path to the file [required]
    pub file_path: String,

    /// Origin hash sum of the file [optional]
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
