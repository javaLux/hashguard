# hashguard

Ensuring the integrity of files through hash sums

```
  ___ ___               .__      ________                       .___
 /   |   \_____    _____|  |__  /  _____/ __ _______ _______  __| _/
/    ~    \__  \  /  ___/  |  \/   \  ___|  |  \__  \\_  __ \/ __ | 
\    Y    // __ \_\___ \|   Y  \    \_\  \  |  // __ \|  | \/ /_/ | 
 \___|_  /(____  /____  >___|  /\______  /____/(____  /__|  \____ | 
       \/      \/     \/     \/        \/           \/           \/ 
```


## Overview
HashGuard is a lean and efficient command-line tool designed to simplify the process of downloading files from the internet and ensuring their integrity using hash sums. With HashGuard, you can confidently download files and verify their authenticity, providing an extra layer of security to your downloads. It is also possible to verify files on the local system.

![Hashguard-Demo](../assets/hashguard_demo.gif?raw=true)

## Features
* **Download files:** You can download a file from the Internet and make sure that they have not been modified during the download process.
* **Local files:** You can also verify a local file with a hash sum
* **Hash Verification:** Verify the authenticity of downloaded or local files by comparing their hash sum with a provided hash value.
* **Support for Various Hash Algorithms:** HashGuard supports different hash algorithms, including SHA-1, SHA2-256, and more. The default Hash-Algorithm is SHA2-256.
* **Intuitive Command-Line Interface:** The simple and user-friendly CLI lets you download and verify files effortlessly.

## Supported OS
* Linux [All common distributions]
* MacOs [Tested on MacOs Monterey]
* Windows 10/11

## Prerequisites
### Installing Rust
To install Rust on your system, just go to the [official Rust website](https://www.rust-lang.org/tools/install), download and install the Rustup toolchain manager.

**Notice:**
Please refer to the installation instructions for your operating system. For certain operating systems, build tools need to be installed before you can use Rust.

## Installation
HashGuard is also available on [crates.io](https://crates.io/crates/hashguard) Rust community's crate registry.
So you can easily install it as binary on your local system.
Use the follow command:
```
cargo install hashguard
```

## Build
To build this project from scratch follow these steps:

* Clone this repository
* Open a terminal
* Navigate to the root directory of the project
* Run the following command
```
cargo build --release
```
* The executable is stored in the _target/release_ folder

## How to use
1. **Command Syntax:**
* ``hashguard download <URL> <HASH_SUM> [OPTIONS]``
* ``hashguard local <FILE_PATH> <HASH_SUM> [OPTIONS]``

2. **Usage Examples:**
* Download a file and verify it with a hash sum by using the default hash algorithm SHA2-256:<br>
``hashguard download https://example.com/file.zip a1b2c3d4e5f6``

* Verify a local file with a hash sum using SHA-1:<br>
``hashguard local /path/to/local_file.txt a1b2c3d4e5f6 -a sha1``

* Use a specific output directory for the downloaded file:<br>
``hashguard download https://example.com/image.jpg a1b2c3d4e5f6 -o /path/to/output_directory``

* Use the --rename option to rename the file to be downloaded:<br>
``hashguard download https://example.com/image.jpg a1b2c3d4e5f6 -r my_fancy_new_file.jpg``

* Get general help:<br>
``hashguard --help``

* Get help on a specific command:<br>
``hashguard download --help``<br>
``hashguard local --help``

3. **Supported Hash Algorithms:**
* MD5
* SHA-1
* SHA2-256
* SHA2-512

## Notice
_**No colored console output under windows?**_<br>
HashGuard of course also works with colored console output (errors = red, hints = yellow, success = green).<br>
If no colored text is displayed in the CMD or PowerShell, instead the ANSI escape sequences before and after an output,<br>
then enabling ANSI escape sequence support may help. Open a CMD or PowerShell as admin and execute following command:<br>
```
reg add HKCU\Console /v VirtualTerminalLevel /t REG_DWORD /d 1
```
This command adds a registry key that enables the conpty feature, which provides ANSI escape sequence support in the Windows console.<br>
Please re-open the terminal and the colored output should work.

## Contributions and Bug Reports
Contributions and bug reports are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request on my GitHub repository.

## License
HashGuard is released under the MIT License.

## Disclaimer
While HashGuard aims to provide reliable file verification, it is essential to exercise caution when downloading files from the internet or using local files for verification. Always ensure that you trust the source and the provided hash sum before proceeding.
