use clap::{Args, Parser, Subcommand};

use crate::verify::Algorithm;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Download a file and verify it with a hash sum
    Download(DownloadArgs),
    /// Verify a local file with a hash sum
    Local(LocalArgs),
}

#[derive(Debug, Args)]
pub struct DownloadArgs {
    /// URL to be used for download the file
    pub url: String,

    /// Origin hash sum of the file
    pub hash_sum: String,

    /// Hash algorithm to be used -> Default is sha2-256
    #[arg(short, long, value_enum)]
    pub algorithm: Option<Algorithm>,

    /// A custom path for the file to be saved (Default is the user download folder)
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<String>,

    /// Rename the file to be downloaded
    #[arg(short, long, value_name = "FILE")]
    pub rename: Option<String>,
}

#[derive(Debug, Args)]
pub struct LocalArgs {
    /// Path to the file
    pub file_path: String,

    /// Origin hash sum of the file
    pub hash_sum: String,

    /// Hash algorithm to be used -> Default is sha2-256
    #[arg(short, long, value_enum)]
    pub algorithm: Option<Algorithm>,
}
