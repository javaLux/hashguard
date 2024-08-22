use std::error::Error;
use std::fmt;
use std::io::{stdout, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use regex::Regex;

use crate::color_templates::WARN_TEMPLATE_NO_BG_COLOR;
use crate::{app, os_specifics};
use color_eyre::eyre::Result;

#[derive(Debug)]
pub enum FilenameError {
    InvalidOnWindows(String),
    InvalidOnUnix(String),
    ReservedFilenameOnWindows,
    EndsWithADot,
}

impl Error for FilenameError {}

impl fmt::Display for FilenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilenameError::InvalidOnWindows(invalid_chars) => {
                write!(f, "Following chars are NOT allowed: {}", invalid_chars)
            }
            FilenameError::InvalidOnUnix(invalid_chars) => {
                write!(f, "Following chars are NOT allowed: {}", invalid_chars)
            }
            FilenameError::ReservedFilenameOnWindows => {
                write!(f, "This is a reserved file name and cannot be used",)
            }
            FilenameError::EndsWithADot => {
                write!(f, "File names under Windows must not end with a dot",)
            }
        }
    }
}

/// Check filename is valid on UNIX based systems
/// Linux/MacOsX forbidden ASCII characters:
/// * \: (colon)
/// * \/ (forward slash)
/// * \\ (backslash)
pub fn is_filename_valid_on_unix(filename: &str) -> bool {
    // Define a regular expression to match valid UNIX filenames
    let regex_invalid_chars = Regex::new(r"^[^:/\\]+$").unwrap();
    regex_invalid_chars.is_match(filename)
}

/// Check if a filename is valid on Windows
/// Window forbidden ASCII characters:
///    * \< (less than)
///    * \> (greater than)
///    * \: (colon - sometimes works, but is actually NTFS Alternate Data Streams)
///    * \" (double quote)
///    * \/ (forward slash)
///    * \\ (backslash)
///    * \| (vertical bar or pipe)
///    * \? (question mark)
///    * (asterisk)
pub fn is_filename_valid_on_windows(filename: &str) -> bool {
    // Define a regular expression to match invalid ASCII characters
    let regex_invalid_chars = Regex::new(r#"^[^<>:"/\\|?*]+$"#).unwrap();

    regex_invalid_chars.is_match(filename)
}

/// Check if the given filename a reserved filename on windows
/// The following filenames are reserved on Windows:
/// CON, PRN, AUX, NUL COM1, COM2, COM3, COM4, COM5, COM6, COM7, COM8, COM9
/// LPT1, LPT2, LPT3, LPT4, LPT5, LPT6, LPT7, LPT8, LPT9
pub fn is_reserved_filename_on_windows(filename: &str) -> bool {
    // Define the regular expression to find reserved filenames, e.g. CON, CON.txt, con.txt will match
    let regex_reserved_filenames =
        Regex::new(r"^(?i:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(\..+)?$").unwrap();

    regex_reserved_filenames.is_match(filename)
}

pub fn validate_filename(os_type: &os_specifics::OS, filename: &str) -> Result<()> {
    match os_type {
        os_specifics::OS::Linux | os_specifics::OS::MacOs => {
            if !is_filename_valid_on_unix(filename) {
                let file_name_err = FilenameError::InvalidOnUnix(
                    os_specifics::UNIX_INVALID_FILE_NAME_CHARS.to_string(),
                );
                return Err(color_eyre::eyre::eyre!(file_name_err.to_string()));
            }
        }
        os_specifics::OS::Windows => {
            // File names under Windows must not end with a dot
            if filename.ends_with('.') {
                return Err(color_eyre::eyre::eyre!(
                    FilenameError::EndsWithADot.to_string()
                ));
            } else {
                // check against reserved filename on windows
                if is_reserved_filename_on_windows(filename) {
                    return Err(color_eyre::eyre::eyre!(
                        FilenameError::ReservedFilenameOnWindows.to_string()
                    ));
                } else if !is_filename_valid_on_windows(filename) {
                    let file_name_err = FilenameError::InvalidOnWindows(
                        os_specifics::WINDOWS_INVALID_FILE_NAME_CHARS.to_string(),
                    );
                    return Err(color_eyre::eyre::eyre!(file_name_err.to_string()));
                }
            }
        }
    }
    Ok(())
}

/// Take a filename over the user input (Input prompt) and check if this is a valid filename
/// dependent on the filename rules of the underlying OS
pub fn enter_and_verify_file_name(os_type: &os_specifics::OS) -> Result<String> {
    let mut file_name = String::new();

    let is_input = Arc::new(AtomicBool::new(true));

    let is_input_clone = is_input.clone();

    // thread to listen on the app state -> if ctrl_c was pressed, quit the app
    let listen_crl_c_thread = std::thread::spawn(move || {
        while is_input.load(Ordering::SeqCst) {
            // listen to the app state
            if !app::APP_SHOULD_RUN.load(Ordering::SeqCst) {
                log::debug!("{} was interrupted by user...", app::APP_NAME);
                println!("{}", app::APP_INTERRUPTED_MSG);
                // terminate app
                std::process::exit(1);
            }
        }
    });

    let result_file_name = loop {
        print!("\t--->: ");
        // to get the input prompt after the 'Enter file name:' without them
        // a new line appears and then follow the input prompt
        stdout().flush()?;

        match std::io::stdin().read_line(&mut file_name) {
            Ok(_) => {
                // remove trailing newline, carriage return and all white spaces
                let file_name_trim = file_name.trim().trim_end_matches(&['\r', '\n'][..]);

                // check if the entered file name is valid for the underlying OS
                match validate_filename(os_type, file_name_trim) {
                    Ok(_) => {
                        is_input_clone.store(false, Ordering::SeqCst);
                        break Ok(file_name_trim.to_string());
                    }
                    Err(filename_err) => {
                        println!(
                            "{} - {}",
                            WARN_TEMPLATE_NO_BG_COLOR.output("Invalid file name"),
                            filename_err
                        );
                        file_name.clear();
                    }
                }
            }
            Err(err) => {
                is_input_clone.store(false, Ordering::SeqCst);
                let err_msg = format!("{:?}", err);
                break Err(color_eyre::eyre::eyre!(err_msg));
            }
        }
    };

    listen_crl_c_thread
        .join()
        .expect("Couldn't join on the 'listen_ctrl_c' thread");

    result_file_name
}
