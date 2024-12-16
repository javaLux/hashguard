use clap::ValueEnum;
use color_eyre::eyre::Result;
use std::path::PathBuf;

use crate::{panic_handling::PanicReport, utils};

pub const APP_NAME: &str = env!("CARGO_CRATE_NAME");

pub const APP_INTERRUPTED_MSG: &str = concat!(
    "\r\x1B[K",
    env!("CARGO_CRATE_NAME"),
    " was interrupted by user..."
);

/// Represents the possible application log level
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Debug,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LogLevel::Debug => {
                write!(f, "debug")
            }
        }
    }
}

/// Define a custom panic hook to handle a application crash.
/// Try to reset the terminal properties in case of the application panicked (crashed).
/// This way, you won't have your terminal messed up if an unexpected error happens.
pub fn initialize_panic_hook(log_level: Option<LogLevel>) -> Result<()> {
    let is_debug_mode = match log_level {
        Some(log_level) => match log_level {
            LogLevel::Debug => true,
        },
        None => false,
    };
    let (_panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .capture_span_trace_by_default(true)
        // show debug info only when app is running in DEBUG mode
        .display_location_section(is_debug_mode)
        .display_env_section(is_debug_mode)
        .into_hooks();
    eyre_hook.install()?;

    // set the custom panic hook handler
    std::panic::set_hook(Box::new(move |panic_info| {
        let mut user_msg = String::new();

        let verbosity_level = if is_debug_mode {
            user_msg.push_str("The application panicked (crashed).");
            better_panic::Verbosity::Full
        } else {
            user_msg.push_str("The application panicked (crashed). Run the application in DEBUG mode [-l debug] to see the full backtrace.");
            better_panic::Verbosity::Minimal
        };

        // print out the Better Panic stacktrace
        better_panic::Settings::new()
            .message(user_msg)
            .most_recent_first(false)
            .lineno_suffix(true)
            .verbosity(verbosity_level)
            .create_panic_handler()(panic_info);

        // write the Crash-Report file
        let log_file_name = format!("{}-Crash-Report.log", APP_NAME);
        let backtrace = std::backtrace::Backtrace::force_capture();
        let panic_report = PanicReport::new(panic_info, backtrace);
        if let Err(report_write_err) =
            panic_report.write_report(&get_data_dir().join(log_file_name))
        {
            log::error!("{}", report_write_err);
        }

        std::process::exit(1);
    }));
    Ok(())
}

/// Initialize the application logging
pub fn initialize_logging(log_level: Option<LogLevel>) -> Result<()> {
    create_data_dir()?;

    if let Some(log_level) = log_level {
        if log_level == LogLevel::Debug {
            set_full_rust_backtrace();
            init_log_writer()?;
            log::debug!(
                "Debug mode is enabled - {} version: {}",
                APP_NAME,
                env!("CARGO_PKG_VERSION")
            );
            log::debug!("Running on => {}", os_info::get())
        }
    }

    Ok(())
}

/// Register the application signal handler.<br>
/// Listens for a termination signal (e.g., `Ctrl+C`) in a background thread to handle user-initiated<br>
/// interruptions gracefully. If interrupted, the application will log the interruption and exit.
pub fn set_ctrl_c_handler() -> Result<()> {
    let exit_cmd = || {
        log::debug!("{} was interrupted by user...", APP_NAME);
        println!("{}", APP_INTERRUPTED_MSG);
        // terminate app
        std::process::exit(1);
    };

    match ctrlc::set_handler(exit_cmd) {
        Ok(_) => Ok(()),
        Err(handler_err) => Err(color_eyre::eyre::eyre!(format!(
            "Failed to set Ctrl-C signal handler - {:?}",
            handler_err
        ))),
    }
}

/// Initializes the verbosity level for the Rust log output based on the specified LogLevel.
///
/// If the provided log level is `LogLevel::Debug`, this function sets the environment
/// variable "RUST_BACKTRACE" to "full", enabling detailed backtrace information in case
/// of a panic. This is particularly useful during debugging to aid in identifying the
/// source of errors.
fn set_full_rust_backtrace() {
    std::env::set_var("RUST_BACKTRACE", "full");
}

/// Initializes the log writer for debugging purposes.
///
/// This function creates a debug log file with a name containing the project name and
/// a timestamp formatted in the "YYYY-MM-DD_HH_MM_SS" format. The log file is stored
/// in the project's data directory. The logging level is set to debug,
/// and the logs which was created by the `log` crate are
/// written to the debug log file using the `simplelog` crate.
fn init_log_writer() -> Result<()> {
    let debug_log_file_name = format!(
        "{}-{}.log",
        APP_NAME,
        chrono::Local::now().format("%Y-%m-%dT%H_%M_%S")
    );
    let debug_file = std::fs::File::create(get_data_dir().join(debug_log_file_name))?;

    let config = simplelog::ConfigBuilder::new()
        .set_time_format_rfc3339()
        .build();
    let log_level = simplelog::LevelFilter::Debug;
    simplelog::WriteLogger::init(log_level, config, debug_file)?;

    Ok(())
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
    let directory = get_data_dir();
    std::fs::create_dir_all(directory.clone())?;
    Ok(())
}

/// Retrieves the data directory path for the project.
///
/// This function uses the `simple_home_dir` crate to determine the user's home directory
/// and constructs the path to the project's data directory within it. If the home directory
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
pub fn get_data_dir() -> PathBuf {
    match simple_home_dir::home_dir() {
        Some(home_dir) => home_dir.join(APP_NAME).join(".data"),
        None => PathBuf::new().join(".").join(APP_NAME).join(".data"),
    }
}

/// Extends the default ``clap --version`` with a custom application version version message
pub fn version() -> String {
    let author = env!("CARGO_PKG_AUTHORS");
    let version = env!("CARGO_PKG_VERSION");
    let repo = env!("CARGO_PKG_REPOSITORY");

    let data_dir_path = utils::get_absolute_path(&get_data_dir());

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
