use anyhow::{Context, Result};
use clap::ValueEnum;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::utils;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

pub const APP_INTERRUPTED_MSG: &str = concat!(
    "\r\x1B[K",
    env!("CARGO_PKG_NAME"),
    " was interrupted by user..."
);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    /// log all available information to the log file
    Debug,
    /// log only necessary information to the log file
    Info,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
        }
    }
}

/// Initialize the application logging
pub fn initialize_logging(log_level: Option<LogLevel>) -> Result<()> {
    create_data_dir()?;

    if let Some(log_level) = log_level {
        init_log_writer(log_level)?;
        match log_level {
            LogLevel::Debug => {
                set_rust_backtrace();
                log::debug!(
                    "{log_level} mode is enabled - {} version: {}",
                    APP_NAME,
                    env!("CARGO_PKG_VERSION")
                );
                log::debug!("Running on => {}", os_info::get());
            }
            LogLevel::Info => {
                log::info!(
                    "{log_level} mode is enabled - {} version: {}",
                    APP_NAME,
                    env!("CARGO_PKG_VERSION")
                );
            }
        }
    }
    Ok(())
}

/// Register the application signal handler.<br>
/// Listens for a termination signal (e.g., `Ctrl+C`) in a background thread to handle user-initiated<br>
/// interruptions gracefully. If interrupted, the application will log the interruption and exit.
pub fn set_ctrl_c_handler() -> Result<()> {
    let exit_cmd = || {
        log::info!("{APP_NAME} was interrupted by user...");
        println!("{APP_INTERRUPTED_MSG}");
        // terminate app
        std::process::exit(1);
    };

    match ctrlc::set_handler(exit_cmd) {
        Ok(_) => Ok(()),
        Err(handler_err) => Err(anyhow::anyhow!(format!(
            "Failed to set Ctrl-C signal handler - {:?}",
            handler_err
        ))),
    }
}

/// Initializes the verbosity level for the Rust log output based on the specified LogLevel.
///
/// If the provided log level is `LogLevel::Debug`, this function sets the environment
/// variable "RUST_BACKTRACE" to "1", enabling detailed backtrace information in case
/// of an error. This is particularly useful during debugging to aid in identifying the
/// source of errors.
pub fn set_rust_backtrace() {
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
}

/// Initializes the log writer for debugging purposes.
///
/// This function creates a debug log file with a name containing the project name and
/// a timestamp formatted in the "YYYY-MM-DD_HH_MM_SS" format. The log file is stored
/// in the project's data directory. The logging level is set to debug,
/// and the logs which was created by the `log` crate are
/// written to the debug log file using the `simplelog` crate.
fn init_log_writer(log_level: LogLevel) -> Result<()> {
    let mut log_file =
        initialize_log_file().with_context(|| "Failed to create application log file")?;

    match log_file.metadata() {
        Ok(metadata) => {
            if metadata.len() > 1 {
                let _ = log_file.write(format!("\n{}\n", "-".repeat(100)).as_bytes());
            }
        }
        Err(_) => {
            let _ = log_file.write(format!("\n{}\n", "-".repeat(100)).as_bytes());
        }
    }

    let config = simplelog::ConfigBuilder::new()
        .set_time_format_rfc3339()
        .build();
    let log_level = match log_level {
        LogLevel::Debug => simplelog::LevelFilter::Debug,
        LogLevel::Info => simplelog::LevelFilter::Info,
    };
    simplelog::WriteLogger::init(log_level, config, log_file)?;

    Ok(())
}

/// Create the log file. If it already exists, make sure it's not over a max
/// size. If it is, move it to a backup path and nuke whatever might be in the
/// backup path.
fn initialize_log_file() -> Result<File> {
    const MAX_FILE_SIZE: u64 = 1000 * 1000; // 1MB
    let path = log_file();

    if fs::metadata(&path).is_ok_and(|metadata| metadata.len() > MAX_FILE_SIZE) {
        // Rename new->old, overwriting old. If that fails, just delete new so
        // it doesn't grow indefinitely. Failure shouldn't stop us from logging
        // though
        let _ = fs::rename(&path, log_file_old()).or_else(|_| fs::remove_file(&path));
    }

    let log_file = OpenOptions::new().create(true).append(true).open(path)?;
    Ok(log_file)
}

/// Get the path to the primary log file. **Parent direct may not exist yet,**
/// caller must create it.
pub fn log_file() -> PathBuf {
    data_dir().join(format!("{APP_NAME}.log"))
}

/// Get the path to the backup log file **Parent direct may not exist yet,**
/// caller must create it.
pub fn log_file_old() -> PathBuf {
    data_dir().join(format!("{APP_NAME}.log.old"))
}

/// Creates the application's data directory.
///
/// This function creates the necessary data directories,
/// if they do not exist.
/// # Returns
///
/// Returns a `Result<()>` if the operation succeeds, or an
/// `Err` variant with an associated `std::io::Error` if any error occurs during the
/// process.
pub fn create_data_dir() -> Result<()> {
    std::fs::create_dir_all(data_dir())
        .with_context(|| "Failed to create application data directory")?;
    Ok(())
}

/// Retrieves the data directory path for the project.
///
/// This function uses the `dirs` crate to determine the user's data directory
/// and constructs the path to the directory dependent on the underlying OS within it. If the home directory
/// is not available, it falls back to a relative path based on the current directory.
///
/// # Returns
///
/// Returns a `PathBuf` representing the data directory path for the project.
///
/// # Note
///
/// Ensure that the `PROJECT_NAME` constant is correctly set before calling this function.
/// The data directory is typically used for storing application-specific data files.
pub fn data_dir() -> PathBuf {
    match dirs::data_dir() {
        Some(data_dir) => data_dir.join(APP_NAME),
        None => PathBuf::new().join(".").join(APP_NAME),
    }
}

/// Extends the default ``clap --version`` with a custom application version version message
pub fn version() -> String {
    let author = env!("CARGO_PKG_AUTHORS");
    let version = env!("CARGO_PKG_VERSION");
    let repo = env!("CARGO_PKG_REPOSITORY");

    let data_dir_path = utils::absolute_path_as_string(&data_dir());

    format!(
        "\
    --- developed with ♥ in Rust
    Authors          : {author}
    Version          : {version}
    Repository       : {repo}

    Data directory   : {data_dir_path}
    "
    )
}
