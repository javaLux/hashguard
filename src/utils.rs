use clap::ValueEnum;
use color_eyre::eyre::Result;
use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};
use url::Url;

use crate::{
    color_templates::{ERROR_TEMPLATE, INFO_TEMPLATE, WARN_TEMPLATE_NO_BG_COLOR},
    commands::CommandResult,
    os_specifics::{self, OS},
    panic_handling::PanicReport,
    verify::Algorithm,
};

pub const PROJECT_NAME: &str = env!("CARGO_CRATE_NAME");

pub const BOUNCING_BAR: [&str; 16] = [
    "[    ]", "[=   ]", "[==  ]", "[=== ]", "[====]", "[ ===]", "[  ==]", "[   =]", "[    ]",
    "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
];

pub const CONTENT_LENGTH_HEADER: &str = "Content-Length";
pub const CONTENT_DISPOSITION_HEADER: &str = "Content-Disposition";

/// Represents the possible application log level
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Info,
    Debug,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LogLevel::Info => {
                write!(f, "info")
            }
            LogLevel::Debug => {
                write!(f, "debug")
            }
        }
    }
}

/// Define a custom panic hook to handle a application crash.
/// Try to reset the terminal properties in case of the application panicked (crashed).
/// This way, you won't have your terminal messed up if an unexpected error happens.
pub fn initialize_panic_hook(log_level: LogLevel) -> Result<()> {
    let is_debug_mode = match log_level {
        LogLevel::Info => false,
        LogLevel::Debug => true,
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

        let verbosity_level = match log_level {
            LogLevel::Info => {
                user_msg.push_str("The application panicked (crashed). Run the application in DEBUG mode [-l debug] to see the full backtrace.");
                better_panic::Verbosity::Minimal
            }
            LogLevel::Debug => {
                user_msg.push_str("The application panicked (crashed).");
                better_panic::Verbosity::Full
            }
        };

        // print out the Better Panic stacktrace
        better_panic::Settings::new()
            .message(user_msg)
            .most_recent_first(false)
            .lineno_suffix(true)
            .verbosity(verbosity_level)
            .create_panic_handler()(panic_info);

        // write the Crash-Report file
        let log_file_name = format!("{}-Crash-Report.log", PROJECT_NAME);
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
pub fn initialize_logging(log_level: LogLevel) -> Result<()> {
    create_data_dir()?;

    if log_level == LogLevel::Debug {
        set_full_rust_backtrace();
        init_log_writer()?;
        log::info!(
            "Debug mode is enabled - {} version: {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
    }
    Ok(())
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
        "{}-Debug-{}.log",
        PROJECT_NAME,
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

/// Processing of the command result
pub fn processing_cmd_result(cmd_result: CommandResult) {
    print_file_location(&cmd_result.file_location);
    log::info!("Calculated hash sum: {}", cmd_result.calculated_hash_sum);

    match cmd_result.hash_compare_result {
        Some(result) => {
            print_verify_result(
                &result.origin_hash_sum,
                &cmd_result.calculated_hash_sum,
                cmd_result.used_algorithm,
                result.is_file_modified,
            );
        }
        None => {
            println!("Calculated hash sum: {}", cmd_result.calculated_hash_sum);
            println!(
                "\n- Used hash algorithm: {}",
                WARN_TEMPLATE_NO_BG_COLOR.output(cmd_result.used_algorithm)
            );
        }
    }
}

fn print_verify_result(
    origin_hash_sum: &str,
    calculated_hash_sum: &str,
    used_algorithm: Algorithm,
    is_file_modified: bool,
) {
    let hash_sum_result = format!(
        "Origin hash sum    : {}\nCalculated hash sum: {}",
        origin_hash_sum, calculated_hash_sum
    );

    println!("{}", hash_sum_result);

    if is_file_modified {
        println!(
            "\n{} - Used hash algorithm: {}",
            ERROR_TEMPLATE.output("Hash sums DO NOT match"),
            WARN_TEMPLATE_NO_BG_COLOR.output(used_algorithm)
        );
    } else {
        println!(
            "\n{} - Used hash algorithm: {}",
            INFO_TEMPLATE.output("Hash sums match"),
            WARN_TEMPLATE_NO_BG_COLOR.output(used_algorithm)
        );
    }
}

/// Prints the passed path as an absolute path, otherwise the passed path
pub fn print_file_location(path: &Path) {
    println!(
        "\n{}      : {}",
        WARN_TEMPLATE_NO_BG_COLOR.output("File location"),
        get_absolute_path(path)
    );
}

/// Gives you the correct time unit dependent on the remaining seconds.
/// Example:
///
/// ````
/// let seconds = 67;
/// let time_unit = calc_duration(seconds);
///
/// assert_eq!(time_unit, "1m 7s".to_string());
/// ````
pub fn calc_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let remaining_seconds = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, remaining_seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, remaining_seconds)
    } else {
        format!("{}s", remaining_seconds)
    }
}

/// Function to check if a given URL is valid and has a path.
/// # Arguments
///
/// url = The url to be parsed ("http://example.com")
///
/// # Returns
///
/// If url is valid -> true, otherwise false
///
/// # Examples
///
/// ````
/// let result = is_url_valid("ThisIsAinvalidUrl");
/// assert_eq!(result, false);
///
/// let result = is_url_valid("http://example.com");
/// assert_eq!(result, true);
/// ````
pub fn is_valid_url(url: &str) -> bool {
    match Url::parse(url) {
        Ok(url) => !url.scheme().is_empty() && url.has_host() && !url.path().is_empty(),
        Err(_) => false,
    }
}

/// Extract the filename of a given url.
/// # Arguments
///
/// url = The url to be parsed ("http://example.com")
///
/// # Returns
///
/// The filename as string slice if it contains valid utf-8 characters,
/// otherwise None.
pub fn extract_file_name_from_url(url: &str) -> Option<&str> {
    match Path::new(url).file_name() {
        Some(file_name) => Some(file_name.to_str()?),
        None => None,
    }
}

/// Try to extract filename from the given Content-Disposition header or the url
pub fn extract_file_name(url: &str, content_disposition: &str, os_type: &OS) -> Option<String> {
    let file_name = if content_disposition.is_empty() {
        match extract_file_name_from_url(url) {
            Some(filename) => {
                // we decode as a precaution
                let filename = decode_percent_encoded_to_utf_8(filename);
                // Remove possible invalid characters for the file name dependent on the underlying os
                Some(replace_invalid_chars_with_underscore(&filename, os_type))
            }
            None => None,
        }
    } else {
        // try to extract filename from Content-Disposition header
        match content_disposition_filename(content_disposition) {
            Some(filename) => {
                let filename = decode_percent_encoded_to_utf_8(&filename);
                Some(replace_invalid_chars_with_underscore(&filename, os_type))
            }
            None => None,
        }
    };

    file_name
}

/// Function to extract filename from Content-Disposition header
pub fn content_disposition_filename(header_value: &str) -> Option<String> {
    let file_name = if !header_value.starts_with("attachment;") {
        None
    } else {
        let parts: Vec<&str> = header_value.split(';').collect();
        if parts.len() < 2 {
            None
        } else {
            let file_name_part = parts.last().unwrap().trim();
            if file_name_part.starts_with("filename*=") {
                // Extract the filename and remove surrounding quotes if present
                if let Some(filename) = file_name_part.strip_prefix("filename*=") {
                    let filename = filename
                        .replace("utf-8", "")
                        .replace("UTF-8", "")
                        .trim_matches(|c| c == '"' || c == '\'')
                        .to_string();
                    return Some(filename);
                } else {
                    None
                }
            } else if file_name_part.starts_with("filename=") {
                if let Some(filename) = file_name_part.strip_prefix("filename=") {
                    let filename = filename
                        .replace("utf-8", "")
                        .replace("UTF-8", "")
                        .trim_matches(|c| c == '"' || c == '\'')
                        .to_string();
                    return Some(filename);
                } else {
                    None
                }
            } else {
                None
            }
        }
    };

    file_name
}

/// Decodes a percent-encoded UTF-8 string.
///
/// This function takes a percent-encoded UTF-8 string as input and decodes it to a valid UTF-8 string.
/// It uses the `percent_encoding` crate to decode percent-encoded characters and handles invalid
/// UTF-8 sequences gracefully. If decoding is successful, the decoded string is returned;
/// otherwise, the original input string is returned. The result is always guaranteed to be a valid UTF-8 string.
///
/// # Arguments
///
/// * `input`: A percent-encoded UTF-8 string that needs to be decoded.
///
/// # Returns
///
/// A `String` containing the decoded UTF-8 string.
///
/// # Examples
///
/// ```
/// let input = "Na%C3%AFve%20file.txt";
/// let result = decode_percent_encoded_to_utf_8(input);
/// assert_eq!(result, "Naïve file.txt");
/// ```
pub fn decode_percent_encoded_to_utf_8(input: &str) -> String {
    percent_encoding::percent_decode_str(input)
        .decode_utf8()
        .unwrap_or(std::borrow::Cow::Borrowed(input))
        .to_string()
}

/// Replaces invalid characters in a file name with underscores based on the specified operating system.
///
/// This function takes a file name and an `OS` enum representing the operating system. It identifies the set
/// of invalid characters for the given operating system and replaces occurrences of these characters with underscores.
/// The result is a sanitized file name suitable for use in the specified OS's file system.
///
/// # Arguments
///
/// * `filename`: The original file name that may contain invalid characters.
/// * `os_type`: An `OS` enum representing the target operating system (Linux, macOS, or Windows).
///
/// # Returns
///
/// A `String` containing the sanitized file name with invalid characters replaced by underscores.
pub fn replace_invalid_chars_with_underscore(filename: &str, os_type: &OS) -> String {
    // Define the set of invalid characters depending on the OS
    let invalid_chars = match os_type {
        OS::Linux | OS::MacOs => os_specifics::UNIX_INVALID_FILE_NAME_CHARS,
        OS::Windows => os_specifics::WINDOWS_INVALID_FILE_NAME_CHARS,
    };

    // Replace invalid characters with underscores
    let sanitized_filename = filename
        .chars()
        .map(|c| if invalid_chars.contains(c) { '_' } else { c })
        .collect::<String>();

    sanitized_filename
}

#[allow(dead_code)]
/// Checks if the HTTP status code indicates a redirection (3xx).
///
/// # Arguments
///
/// * `status_code`: The HTTP status code to check.
///
/// # Returns
///
/// `true` if the status code indicates a redirection, otherwise `false`.
pub fn is_redirection(status_code: u16) -> bool {
    matches!(status_code, 301 | 302 | 307 | 308)
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
        Some(home_dir) => home_dir.join(PROJECT_NAME).join(".data"),
        None => PathBuf::new().join(".").join(PROJECT_NAME).join(".data"),
    }
}

/// Return the passed path as an absolute path, otherwise the passed path
pub fn get_absolute_path(path: &Path) -> String {
    match path.absolutize() {
        Ok(absolute_path) => absolute_path.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

/// Get a specific version info string
pub fn version() -> String {
    let author = clap::crate_authors!();
    let version = clap::crate_version!();

    let data_dir_path = get_absolute_path(&get_data_dir());

    format!(
        "\
    --- developed with ♥ in Rust
    Authors                 : {author}
    Version                 : {version}

    Data directory          : {data_dir_path}
    "
    )
}
