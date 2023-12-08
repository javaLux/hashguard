use std::path::Path;
use url::Url;

use crate::{
    color_templates::{ERROR_TEMPLATE, INFO_TEMPLATE},
    verify::Algorithm,
};

/// Print the user output dependent on, if the file to check was modified or not and the used hash sum algorithm
pub fn generate_output(is_file_modified: bool, hash_alg: Option<Algorithm>) {
    let hash_alg = match hash_alg {
        Some(alg) => alg,
        None => Algorithm::SHA2_256,
    };

    if !is_file_modified {
        println!(
            "\n{} - Used hash algorithm: {:?}",
            INFO_TEMPLATE.output("Hash sum match"),
            hash_alg
        );
    } else {
        println!(
            "\n{} - Used hash algorithm: {:?}",
            ERROR_TEMPLATE.output("Hash sum do not match"),
            hash_alg
        );
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_url() {
        let test_url = "https://example.com/files/document.pdf";

        let result = is_valid_url(test_url);

        assert_eq!(result, true);
    }

    #[test]
    fn test_invalid_url() {
        let test_url = "HelloWorld";

        let result = is_valid_url(test_url);

        assert_eq!(result, false);
    }

    #[test]
    fn test_extract_filename() {
        let test_url = "https://example.com/files/document.pdf";

        let result = extract_file_name_from_url(test_url);

        assert_eq!(result, Some("document.pdf"));
    }
}
