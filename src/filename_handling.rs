use std::io::{stdout, Write};

use regex::Regex;

use crate::color_templates::{ERROR_TEMPLATE, WARN_TEMPLATE_NO_BG_COLOR};
use crate::os_specifics;

// const for display the forbidden filename chars dependent on the underlying OS
const UNIX_INVALID_FILE_NAME_CHARS: &str = r":/\\";
const WINDOWS_INVALID_FILE_NAME_CHARS: &str = r#"<>:"/\\|?*"#;

/// Check filename is valid on UNIX based systems
/// Linux/MacOsX forbidden ASCII characters:
/// : (colon)
/// / (forward slash)
/// \ (backslash)
fn is_filename_valid_on_unix(filename: &str) -> bool {
    // Define a regular expression to match valid UNIX filenames
    let regex_invalid_chars = Regex::new(r"^[^:/\\]+$").unwrap();
    regex_invalid_chars.is_match(filename)
}

/// Check if a filename is valid on Windows
/// Window forbidden ASCII characters:
///    < (less than)
///    > (greater than)
///    : (colon - sometimes works, but is actually NTFS Alternate Data Streams)
///    " (double quote)
///    / (forward slash)
///    \ (backslash)
///    | (vertical bar or pipe)
///    ? (question mark)
///    * (asterisk)
fn is_filename_valid_on_windows(filename: &str) -> bool {
    // Define a regular expression to match invalid ASCII characters
    let regex_invalid_chars = Regex::new(r#"^[^<>:"/\\|?*]+$"#).unwrap();

    regex_invalid_chars.is_match(filename)
}

/// Check if the given filename a reserved filename on windows
/// The following filenames are reserved on Windows:
/// CON, PRN, AUX, NUL COM1, COM2, COM3, COM4, COM5, COM6, COM7, COM8, COM9
/// LPT1, LPT2, LPT3, LPT4, LPT5, LPT6, LPT7, LPT8, LPT9
fn is_reserved_filename_on_windows(filename: &str) -> bool {
    // Define the regular expression to find reserved filenames, e.g. CON, CON.txt, con.txt will match
    let regex_reserved_filenames =
        Regex::new(r"^(?i:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(\..+)?$").unwrap();

    regex_reserved_filenames.is_match(filename)
}

pub fn is_valid_filename(os_type: &os_specifics::OS, filename: &str) -> bool {
    match os_type {
        os_specifics::OS::Linux | os_specifics::OS::MacOsX => is_filename_valid_on_unix(filename),
        os_specifics::OS::Windows => {
            // File names under Windows cannot end with a dot
            if filename.ends_with('.') {
                false
            } else {
                // check against reserved filename on windows
                if is_reserved_filename_on_windows(filename) {
                    false
                } else {
                    // test if filename contains invalid chars
                    is_filename_valid_on_windows(filename)
                }
            }
        }
    }
}

/// Take a filename over the user input (Input prompt) and check if this is a valid filename
/// dependent on the filename rules of the underlying OS
pub fn enter_and_verify_file_name(os_type: &os_specifics::OS) -> String {
    let mut file_name = String::new();

    loop {
        print!("Enter file name: ");
        // to get the input prompt after the 'Enter file name:' without them
        // a new line appears and then follow the input prompt
        stdout().flush().unwrap();

        match std::io::stdin().read_line(&mut file_name) {
            Ok(_) => {
                // remove trailing newline, carriage return and all white spaces
                let file_name_trim = file_name.trim().trim_end_matches(&['\r', '\n'][..]);

                // check if the entered file name is valid for the underlying OS
                match os_type {
                    os_specifics::OS::Linux | os_specifics::OS::MacOsX => {
                        if is_filename_valid_on_unix(file_name_trim) {
                            file_name = file_name_trim.to_string();
                            break;
                        } else {
                            println!(
                                "{} - Following chars are NOT allowed: {}",
                                WARN_TEMPLATE_NO_BG_COLOR.output("Invalid file name"),
                                ERROR_TEMPLATE.output(UNIX_INVALID_FILE_NAME_CHARS)
                            );
                            file_name.clear();
                        }
                    }
                    os_specifics::OS::Windows => {
                        // File names under Windows cannot end with a dot,
                        // so this is removed as a precautionary measure
                        let file_name_trim = file_name_trim.trim_end_matches(&['.']);

                        // check against reserved filename on windows
                        if is_reserved_filename_on_windows(file_name_trim) {
                            println!(
                                "{} - This is a reserved filename and can not be used",
                                WARN_TEMPLATE_NO_BG_COLOR.output("Invalid file name")
                            );
                            file_name.clear();
                        } else {
                            // test if filename contains invalid chars
                            if is_filename_valid_on_windows(file_name_trim) {
                                file_name = file_name_trim.to_string();
                                break;
                            } else {
                                println!(
                                    "{} - Following chars are NOT allowed: {}",
                                    WARN_TEMPLATE_NO_BG_COLOR.output("Invalid file name"),
                                    ERROR_TEMPLATE.output(WINDOWS_INVALID_FILE_NAME_CHARS)
                                );
                                file_name.clear();
                            }
                        }
                    }
                }
            }
            Err(err) => {
                println!(
                    "{}: {}",
                    ERROR_TEMPLATE.output("Failed to read from STDIN"),
                    err
                );
                std::process::exit(1);
            }
        }
    }

    file_name
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_filenames_windows() {
        let filename1 = "valid_filename.txt";
        let filename2 = "test/filename.txt";
        let filename3 = "file?name.pdf";
        let filename4 = "filename\\.csv";

        assert!(is_filename_valid_on_windows(filename1));
        assert!(!is_filename_valid_on_windows(filename2));
        assert!(!is_filename_valid_on_windows(filename3));
        assert!(!is_filename_valid_on_windows(filename4));
    }

    #[test]
    fn test_reserved_filename_on_windows() {
        let reserved_filenames = vec![
            "CON",
            "PRN",
            "AUX",
            "NUL",
            "COM1",
            "COM2",
            "COM3",
            "COM4",
            "COM5",
            "COM6",
            "COM7",
            "COM8",
            "COM9",
            "LPT1",
            "LPT2",
            "LPT3",
            "LPT4",
            "LPT5",
            "LPT6",
            "LPT7",
            "LPT8",
            "LPT9",
            "CON.txt",
            "PRN.docs",
            "AUX.toml",
            "NUL.cu",
            "COM1.bin",
            "COM2.test",
            "COM3.zip",
            "COM4.7z",
            "COM5.op",
            "COM6.exe",
            "COM7.sh",
            "COM8.rs",
            "COM9.lil",
        ];

        for filename in reserved_filenames {
            assert!(is_reserved_filename_on_windows(filename));
        }
    }

    #[test]
    fn trim_dot_from_end() {
        let test_string = "Hello world.";
        let result = test_string.trim_end_matches(&['.']);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_filenames_unix() {
        let filename1 = "valid_filename";
        let filename2 = "test/filename.txt";
        let filename3 = "file:name.pdf";
        let filename4 = "filename\\";

        assert!(is_filename_valid_on_unix(filename1));
        assert!(!is_filename_valid_on_unix(filename2));
        assert!(!is_filename_valid_on_unix(filename3));
        assert!(!is_filename_valid_on_windows(filename4));
    }
}
