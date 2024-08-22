use path_absolutize::Absolutize;
use std::path::Path;
use url::Url;

use crate::{
    color_templates::{ERROR_TEMPLATE, INFO_TEMPLATE, WARN_TEMPLATE_NO_BG_COLOR},
    commands::CommandResult,
    os_specifics::{self, OS},
    verify::Algorithm,
};

pub const BOUNCING_BAR: [&str; 16] = [
    "[    ]", "[=   ]", "[==  ]", "[=== ]", "[====]", "[ ===]", "[  ==]", "[   =]", "[    ]",
    "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
];

// const for the calculation of the total file size in a human readable format
const KIB: f64 = 1024.0;
const MIB: f64 = KIB * KIB;
const GIB: f64 = KIB * MIB;
const TIB: f64 = KIB * GIB;

/// Processing of the command result
pub fn processing_cmd_result(cmd_result: CommandResult) {
    print_file_location(&cmd_result.file_location);
    let calculated_hash_output = format!("Calculated hash sum: {}", cmd_result.calculated_hash_sum);
    log::debug!("{calculated_hash_output}");

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
            println!("{calculated_hash_output}");
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
    } else if remaining_seconds < 1 {
        "< 1s".to_string()
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
/// ```
/// let result = is_url_valid("ThisIsAinvalidUrl");
/// assert!(!result);
///
/// let result = is_url_valid("http://example.com");
/// assert!(result);
/// ```
pub fn is_valid_url(url: &str) -> bool {
    match Url::parse(url) {
        Ok(url) => {
            !url.scheme().is_empty()
                && (url.scheme() == "http" || url.scheme() == "https")
                && url.has_host()
                && !url.path().is_empty()
        }
        Err(_) => false,
    }
}

/// Extracts the file name from the provided URL.
///
/// This function parses the given URL using the `url` crate, extracting the last segment
/// of the URL path, which typically represents the file name. If the URL is successfully
/// parsed and a non-empty file name segment is found, it is returned as an `Option<String>`.
///
/// # Arguments
///
/// * `url` - A string slice representing the URL from which to extract the file name.
///
/// # Returns
///
/// An `Option` containing the extracted file name as a `String`, if found. If the URL
/// cannot be parsed or no file name is present, `None` is returned.
///
/// # Example
///
/// ```
/// let url = "https://example.com/path/to/file.txt?page=2";
/// assert_eq!(extract_file_name_from_url(url), Some("file.txt".to_string()));
///
/// let invalid_url = "not_a_url";
/// assert_eq!(extract_file_name_from_url(invalid_url), None);
/// ```
///
/// # Note
/// If the url contains any query parameters (URL parameters) these are automatically removed from
/// the last URL path segment.
/// This function does not modify the original URL string.
pub fn extract_file_name_from_url(url: &str) -> Option<String> {
    match Url::parse(url) {
        Ok(url) => {
            let file_name = url.path().split('/').collect::<Vec<&str>>();
            match file_name.last() {
                Some(file_name) => {
                    if !file_name.is_empty() {
                        let file_name = file_name.to_string();
                        Some(file_name)
                    } else {
                        None
                    }
                }
                None => None,
            }
        }
        Err(_) => None,
    }
}

/// Try to extract filename from the given Content-Disposition header or the url
pub fn extract_file_name(url: &str, content_disposition: &str, os_type: &OS) -> Option<String> {
    if content_disposition.is_empty() {
        match extract_file_name_from_url(url) {
            Some(filename) => {
                // we decode as a precaution
                let filename = decode_percent_encoded_to_utf_8(&filename);
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
    }
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

/// Return the passed path as an absolute path, otherwise the passed path
pub fn get_absolute_path(path: &Path) -> String {
    match path.absolutize() {
        Ok(absolute_path) => absolute_path.display().to_string(),
        Err(_) => path.display().to_string(),
    }
}

/// Converts a given size in bytes into a human-readable format.
///
/// # Arguments
///
/// * `bytes` - The size in bytes to be converted into a human-readable format.
///
/// # Returns
///
/// A string representing the human-readable format of the given size.
///
/// # Examples
///
/// ```
///  let size_in_bytes: usize = 2048;
///  let readable_size = convert_bytes_to_human_readable(size_in_bytes);
///  assert_eq!("2.00 KiB".to_string(), readable_size);
/// ```
pub fn convert_bytes_to_human_readable(bytes: usize) -> String {
    if bytes < KIB as usize {
        format!("{} B", bytes)
    } else if bytes < MIB as usize {
        format!("{:.2} KiB", bytes as f64 / KIB)
    } else if bytes < GIB as usize {
        format!("{:.2} MiB", bytes as f64 / MIB)
    } else if bytes < TIB as usize {
        format!("{:.2} GiB", bytes as f64 / GIB)
    } else {
        format!("{:.2} TiB", bytes as f64 / TIB)
    }
}
