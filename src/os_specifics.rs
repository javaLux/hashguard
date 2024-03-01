use std::path::PathBuf;

use crate::utils;

const LINUX: &str = "linux";
const MAC_OS: &str = "macos";
const WINDOWS: &str = "windows";

// forbidden filename chars dependent on the underlying OS
pub const UNIX_INVALID_FILE_NAME_CHARS: &str = r":/\\";
pub const WINDOWS_INVALID_FILE_NAME_CHARS: &str = r#"<>:"/\\|?*"#;

/// Supported Operating-Systems
#[derive(Debug, PartialEq, PartialOrd)]
pub enum OS {
    Linux,
    MacOs,
    Windows,
}

/// Get the correct os type of the underlying OS.
pub fn get_os() -> Option<OS> {
    // get os string
    let os_name = std::env::consts::OS;

    if os_name.eq_ignore_ascii_case(LINUX) {
        Some(OS::Linux)
    } else if os_name.eq_ignore_ascii_case(MAC_OS) {
        Some(OS::MacOs)
    } else if os_name.eq_ignore_ascii_case(WINDOWS) {
        Some(OS::Windows)
    } else {
        None
    }
}

/// Retrieves the default download directory path.
///
/// This function uses the `simple_home_dir` crate to determine the user's home directory
/// and constructs the path to the default download folder. If the home directory
/// is not available, it falls back to a relative path based on the current directory.
///
/// # Returns
///
/// Returns a `PathBuf` representing the download directory path.
pub fn get_default_download_folder() -> PathBuf {
    match simple_home_dir::home_dir() {
        Some(home_dir) => home_dir.join("Downloads"),
        None => PathBuf::new()
            .join(".")
            .join(utils::PROJECT_NAME)
            .join("Downloads"),
    }
}
