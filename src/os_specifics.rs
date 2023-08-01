use std::path::PathBuf;

const LINUX: &str = "linux";
const MAC_OS: &str = "macos";
const WINDOWS: &str = "windows";

const UNIX_USER_ENV: &str = "HOME";
const WINDOWS_USER_ENV: &str = "USERPROFILE";

/// Supported Operating-Systems
#[derive(Debug, PartialEq, PartialOrd)]
pub enum OS {
    Linux,
    MacOsX,
    Windows,
}

/// Get the correct os type of the underlying OS.
pub fn get_os() -> Option<OS> {
    // get os string
    let os_name = std::env::consts::OS;

    if os_name.eq_ignore_ascii_case(LINUX) {
        Some(OS::Linux)
    } else if os_name.eq_ignore_ascii_case(MAC_OS) {
        Some(OS::MacOsX)
    } else if os_name.eq_ignore_ascii_case(WINDOWS) {
        Some(OS::Windows)
    } else {
        None
    }
}

/// Get the correct path to the user download folder,
/// depending on the underlying os.
/// Return None if the OS is unsupported or the user profile env is not set.
pub fn get_default_download_folder(os_type: &OS) -> Option<String> {
    match os_type {
        OS::Linux | OS::MacOsX => match std::env::var(UNIX_USER_ENV) {
            Ok(home) => Some(get_os_specific_path(&home)),
            _ => None,
        },
        OS::Windows => match std::env::var(WINDOWS_USER_ENV) {
            Ok(home) => Some(get_os_specific_path(&home)),
            _ => None,
        },
    }
}

/// Get the correct path to the user download folder dependent on the underlying platform.
/// Handles the platform-specific path separator and ensure
/// that the file path is valid for the target operating system.
fn get_os_specific_path(path: &str) -> String {
    let mut path_buf = PathBuf::new();
    path_buf.push(path);
    path_buf.push("Downloads");
    path_buf.to_string_lossy().to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_os_type() {
        let os = get_os();

        match os {
            Some(OS::Linux) => assert_eq!(Some(OS::Linux), os),
            Some(OS::MacOsX) => assert_eq!(Some(OS::MacOsX), os),
            Some(OS::Windows) => assert_eq!(Some(OS::Windows), os),
            None => assert_eq!(None, os),
        }
    }
}
