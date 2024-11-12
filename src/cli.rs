use std::path::PathBuf;

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
    /// Download a file and calculate a hash sum
    Download(DownloadArgs),
    /// Calculate a hash sum from a file/dir or a byte buffer
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
    pub output: Option<PathBuf>,

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
    #[arg(
        short,
        long,
        conflicts_with = "buffer",
        help = "Path to a file/dir for which the hash sum will be calculated",
        value_name = "PATH"
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
