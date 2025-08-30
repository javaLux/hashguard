use anyhow::Result;
use path_absolutize::Absolutize;
use regex::Regex;
use std::path::Path;
use url::Url;

use crate::{
    app,
    color_templates::{ERROR_TEMPLATE, INFO_TEMPLATE, WARN_TEMPLATE_NO_BG_COLOR},
    command_handling::{CommandResult, HashCompareResult},
    hasher::Algorithm,
    os_specifics::{self, OS},
};

pub const CAPACITY: usize = 64 * 1024;

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
pub fn processing_cmd_result(cmd_result: &CommandResult) -> Result<()> {
    let hash_source = match &cmd_result.file_location {
        Some(file_location) => absolute_path_as_string(file_location),
        None => match &cmd_result.buffer {
            Some(buffer) => format!("Buffer of size {} byte(s)", buffer.len()),
            None => "Buffer of unknown size".to_string(),
        },
    };

    println!(
        "\n{}   : {}",
        WARN_TEMPLATE_NO_BG_COLOR.output("Input source"),
        hash_source
    );

    print_hash_result(
        cmd_result.hash_compare_result.as_ref(),
        cmd_result.used_algorithm,
        &cmd_result.calculated_hash_sum,
    );

    save_calculated_hash_sum(cmd_result)?;
    Ok(())
}

/// Print and log the hash result
fn print_hash_result(
    hash_to_compare: Option<&HashCompareResult>,
    used_algorithm: Algorithm,
    calculated_hash_sum: &str,
) {
    let calculated_hash_sum = format!("Calculated hash: {calculated_hash_sum}");

    log::info!("{calculated_hash_sum}");
    println!("{calculated_hash_sum}");

    if let Some(hash_to_compare) = hash_to_compare {
        let origin_hash = format!(
            "Given hash     : {}",
            hash_to_compare.origin_hash_sum.to_ascii_lowercase()
        );

        log::info!("{origin_hash}");
        println!("{origin_hash}");

        if !hash_to_compare.is_hash_equal {
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
    } else {
        println!(
            "\n- Used hash algorithm: {}",
            WARN_TEMPLATE_NO_BG_COLOR.output(used_algorithm)
        );
    }
}

fn save_calculated_hash_sum(cmd_result: &CommandResult) -> Result<()> {
    if cmd_result.save {
        let app_data_dir = app::data_dir();
        let (file_name, content) = if let Some(file_path) = &cmd_result.file_location {
            let prefix = file_path
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("hash_sum"))
                .to_string_lossy();
            (
                format!(
                    "{}.{}",
                    prefix,
                    cmd_result.used_algorithm.to_string().to_lowercase()
                ),
                format!("{}\t{}", cmd_result.calculated_hash_sum, prefix),
            )
        } else {
            // If a buffer was hashed, use a default file name
            (
                format!(
                    "hash_sum.{}",
                    cmd_result.used_algorithm.to_string().to_lowercase()
                ),
                format!(
                    "{}\t{}",
                    cmd_result.calculated_hash_sum,
                    cmd_result.buffer.as_deref().unwrap_or_default()
                ),
            )
        };
        let hash_sum_file_path = app_data_dir.join(file_name);
        std::fs::write(hash_sum_file_path, content)?;
    }
    Ok(())
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
        format!("{hours}h {minutes}m {remaining_seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {remaining_seconds}s")
    } else if remaining_seconds < 1 {
        "< 1s".to_string()
    } else {
        format!("{remaining_seconds}s")
    }
}

/// Function to check if a given URL is valid or not.
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
                && (matches!(url.scheme(), "http" | "https"))
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
    Url::parse(url)
        .ok()?
        .path()
        .split('/')
        .next_back()
        .and_then(|file_name| {
            if file_name.trim().is_empty() {
                None
            } else {
                Some(file_name.to_string())
            }
        })
}

/// Try to extract the filename from the server response
pub fn extract_file_name(url: &str, content_disposition: &str, os_type: &OS) -> Option<String> {
    // Attempt to extract the filename from Content-Disposition or fallback to the URL path
    let filename = extract_filename_from_content_disposition(content_disposition)
        .or_else(|| extract_file_name_from_url(url));

    // If a filename is found, process it
    filename
        .map(|f| decode_percent_encoded_to_utf_8(&f))
        .map(|f| replace_invalid_chars_with_underscore(&f, os_type))
}

/// Function to extract filename from Content-Disposition header
pub fn extract_filename_from_content_disposition(header_value: &str) -> Option<String> {
    if !header_value.to_lowercase().starts_with("attachment;") || header_value.trim().is_empty() {
        return None;
    }

    let filename_prefixes = ["filename*=", "filename="];
    let utf8_regex = Regex::new(r"(?i)utf-8").unwrap(); // Case-insensitive regex for "utf-8"

    for part in header_value.split(';').map(str::trim) {
        for prefix in &filename_prefixes {
            if let Some(filename) = part.strip_prefix(prefix) {
                let filename = utf8_regex
                    .replace_all(filename, "")
                    .trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '"' | '\''))
                    .to_string();
                if !filename.is_empty() {
                    return Some(filename);
                }
            }
        }
    }

    None
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
    filename
        .chars()
        .map(|c| if invalid_chars.contains(c) { '_' } else { c })
        .collect::<String>()
}

/// Return the passed path as an absolute path, otherwise the passed path
pub fn absolute_path_as_string(path: &Path) -> String {
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
        format!("{bytes} B")
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

#[cfg(test)]
mod test {
    use super::*;
    use os_specifics::OS;

    #[test]
    fn test_valid_url_1() {
        let test_url = "http://example.com/files/document.pdf";

        assert!(is_valid_url(test_url));
    }

    #[test]
    fn test_valid_url_2() {
        let test_url = "https://google.de";

        assert!(is_valid_url(test_url));
    }

    #[test]
    fn test_invalid_url_1() {
        let test_url = "HelloWorld";

        assert!(!is_valid_url(test_url));
    }

    #[test]
    fn test_invalid_url_2() {
        let test_url = "file://tmp/foo";

        assert!(!is_valid_url(test_url));
    }

    #[test]
    fn test_invalid_url_3() {
        let test_url = "www.example.com";
        assert!(!is_valid_url(test_url));
    }

    #[test]
    fn test_extract_filename_from_url_1() {
        let test_url = "https://example.com/files/document.pdf";

        let result = extract_file_name_from_url(test_url);

        assert_eq!(result, Some("document.pdf".to_string()));
    }

    #[test]
    fn test_extract_filename_from_url_2() {
        let test_url = "http://blah.com/path1/path2/test_file.txt?a=1&b=2";

        let result = extract_file_name_from_url(test_url);

        assert_eq!(result, Some("test_file.txt".to_string()));
    }

    #[test]
    fn test_extract_filename_from_url_3() {
        let test_url = "https://google.de/";

        let result = extract_file_name_from_url(test_url);

        assert_eq!(result, None);
    }

    #[test]
    fn test_basic_case() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; filename=\"example.txt\""),
            Some("example.txt".to_string())
        );
    }

    #[test]
    fn test_case_insensitive_attachment() {
        assert_eq!(
            extract_filename_from_content_disposition("Attachment; filename=\"example.txt\""),
            Some("example.txt".to_string())
        );
    }

    #[test]
    fn test_filename_with_utf8_encoding() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; filename*=utf-8''example.txt"),
            Some("example.txt".to_string())
        );
    }

    #[test]
    fn test_filename_with_utf8_encoding_uppercase() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; filename*=UTF-8''example.txt"),
            Some("example.txt".to_string())
        );
    }

    #[test]
    fn test_filename_with_quotes() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; filename=\"example.txt\""),
            Some("example.txt".to_string())
        );
    }

    #[test]
    fn test_filename_with_single_quotes() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; filename='example.txt'"),
            Some("example.txt".to_string())
        );
    }

    #[test]
    fn test_filename_with_extra_spaces() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; filename=   \"example.txt\"   "),
            Some("example.txt".to_string())
        );
    }

    #[test]
    fn test_filename_with_special_characters() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; filename=\"example@123.txt\""),
            Some("example@123.txt".to_string())
        );
    }

    #[test]
    fn test_empty_filename() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; filename=\"\""),
            None
        );
    }

    #[test]
    fn test_no_filename_1() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment;"),
            None
        );
    }

    #[test]
    fn test_no_filename_2() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; other_param=test"),
            None
        );
    }

    #[test]
    fn test_invalid_header() {
        assert_eq!(
            extract_filename_from_content_disposition("inline; filename=\"example.txt\""),
            None
        );
    }

    #[test]
    fn test_multiple_parts_filename_not_last() {
        assert_eq!(
            extract_filename_from_content_disposition(
                "attachment; something; filename=\"example.txt\""
            ),
            Some("example.txt".to_string())
        );
    }

    #[test]
    fn test_multiple_parts_filename_star_not_last() {
        assert_eq!(
            extract_filename_from_content_disposition(
                "attachment; something; filename*=utf-8''example.txt"
            ),
            Some("example.txt".to_string())
        );
    }

    #[test]
    fn test_filename_with_mixed_case() {
        assert_eq!(
            extract_filename_from_content_disposition("attachment; filename=\"Example.TXT\""),
            Some("Example.TXT".to_string())
        );
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_empty_input() {
        let input = "";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_no_encoding() {
        let input = "example_filename.txt";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "example_filename.txt");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_single_encoding() {
        let input = "file%20with%20spaces.txt";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "file with spaces.txt");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_multiple_encodings_1() {
        let input = "file%20with%20spaces%20and%20special%21%23%25.txt";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "file with spaces and special!#%.txt");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_multiple_encodings_2() {
        let input = "Na%C3%AFve%20file.txt";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "Naïve file.txt");
    }

    #[test]
    fn test_decode_percent_encoded_to_utf_invalid_encoding() {
        // Invalid percent encoding, should be treated as plain text
        let input = "invalid%2xencoding";
        let result = decode_percent_encoded_to_utf_8(input);
        assert_eq!(result, "invalid%2xencoding");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_linux() {
        let filename = "my:file/with\\invalid\\characters.txt";
        let os_type = OS::Linux;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "my_file_with_invalid_characters.txt");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_macos() {
        let filename = "my:file/with\\invalid\\characters.txt";
        let os_type = OS::MacOs;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "my_file_with_invalid_characters.txt");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_windows() {
        let filename = "my?file*with<invalid>characters\\fancy:style.txt";
        let os_type = OS::Windows;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "my_file_with_invalid_characters_fancy_style.txt");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_no_replacement() {
        let filename = "file_without_invalid_characters.txt";
        let os_type = OS::Linux;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "file_without_invalid_characters.txt");
    }

    #[test]
    fn test_replace_invalid_chars_with_underscore_empty_filename() {
        let filename = "";
        let os_type = OS::Windows;
        let result = replace_invalid_chars_with_underscore(filename, &os_type);
        assert_eq!(result, "");
    }
}
