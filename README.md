# hashguard
[![Build Status](https://github.com/javaLux/hashguard/actions/workflows/rust.yml/badge.svg)](https://github.com/javaLux/hashguard/actions)
[![dependency status](https://deps.rs/repo/github/javaLux/hashguard/status.svg)](https://deps.rs/repo/github/javaLux/hashguard)
[![GitHub license](https://img.shields.io/github/license/javaLux/hashguard.svg)](https://github.com/javaLux/hashguard/blob/main/LICENSE)
[![crates.io](https://img.shields.io/crates/v/hashguard.svg)](https://crates.io/crates/hashguard)
![maintenance-status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

Ensuring the integrity of files through hash sums

```
  ___ ___               .__      ________                       .___
 /   |   \_____    _____|  |__  /  _____/ __ _______ _______  __| _/
/    ~    \__  \  /  ___/  |  \/   \  ___|  |  \__  \\_  __ \/ __ | 
\    Y    // __ \_\___ \|   Y  \    \_\  \  |  // __ \|  | \/ /_/ | 
 \___|_  /(____  /____  >___|  /\______  /____/(____  /__|  \____ | 
       \/      \/     \/     \/        \/           \/           \/ 
```


# Overview
HashGuard is a lean and efficient command-line tool designed to simplify the process of downloading files from the internet and ensuring their integrity using hash sums. With HashGuard, you can confidently download files and verify their authenticity, providing an extra layer of security to your downloads. It is also possible to verify files on the local system.

![Hashguard-Demo](../assets/hashguard_demo.gif?raw=true)

## Features
* **Download:**
  * Download a file and have a specific hash sum calculated depending on the selected hash algorithm
  * Or you can directly enter a known hash sum to compare it after the download.
    This allows you to check whether the file was changed during the download process
  * **_Notice_**
    * If you use the download command, please enclose the URL in double quotation marks.
      Because by enclosing the URL in double quotation marks, you tell the shell to treat the entire string as a single argument, even if it contains spaces or other special characters. This can prevent errors and unexpected behavior in your shell.
  
* **Local:**
  * As described in the download feature, hash sums can also be calculated from local files or a known hash sum can be specified to compare them

* **Hash Verification:** Verify the authenticity of downloaded or local files by comparing their hash sum with a provided hash value.
* **Support for Various Hash Algorithms:** HashGuard supports different hash algorithms, including SHA-1, SHA2-256, and more. The default Hash-Algorithm is SHA2-256.
* **Intuitive Command-Line Interface:** The simple and user-friendly CLI lets you easily calculate and compare hash sums.
* **Log-Level**
  * You can enable the application debug mode
    * This provides additional functionality such as logging all relevant operations in a log file.
    * Furthermore, the debug mode can be useful to get detailed information about an error (e.g. display a full backtrace)

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
### Using Natively Compiled Binaries
If you don't have Rust installed or prefer not to build the project yourself, you can use the precompiled binaries provided in the [Releases](https://github.com/javaLux/hashguard/releases) section. Download the appropriate binary for your operating system and architecture, and you're good to go!

- [Download the latest release](https://github.com/javaLux/hashguard/releases/latest)

## Build the project
To build this project from scratch follow these steps:

* Clone this repository
* Open a terminal
* Navigate to the root directory of the project
* Run the following command
```
cargo build --release
```
* The compiled binary will be available at `target/release/`

## How to use
### General Syntax
* ``hashguard [OPTIONS] <COMMAND>``

### Command specific syntax
* ``hashguard [OPTIONS] download <URL> [HASH_SUM] [OPTIONS]``
* ``hashguard [OPTIONS] local <FILE_PATH> [HASH_SUM] [OPTIONS]``

### Usage Examples
* Download a file and verify it with a hash sum by using the default hash algorithm SHA2-256:
  ````shell
  hashguard download "https://example.com/file.zip" a1b2c3d4e5f6
  ````

* Download a file and calculate a hash sum with a specific hash algorithm:
  ````shell
  hashguard download "https://example.com/file.zip" -a sha2-512
  ````

* Verify a local file with a hash sum using SHA-1:
  ````shell
  hashguard local /path/to/local_file.txt a1b2c3d4e5f6 -a sha1
  ````

* Calculate a hash sum from a local file with the default hash algorithm:
  ````shell
  hashguard local /path/to/local_file.txt
  ````

* Use a specific output directory for the downloaded file:
  ````shell
  hashguard download "https://example.com/image.jpg" a1b2c3d4e5f6 -o /path/to/output_directory
  ````

* Use the --rename option to rename the file to be downloaded:
  ````shell
  hashguard download "https://example.com/image.jpg" a1b2c3d4e5f6 -r "my_fancy_new_file.jpg"
  ````

* Enable the debug log level:
  ````shell
  hashguard -l debug download "https://example.com/file.zip" a1b2c3d4e5f6
  ````
  * In the event of an error, a full backtrace is displayed
  * In addition, all log outputs are saved in a log file in the application's data directory.
  * You can find out the application data directory with the [--version] command
  * Please note that the application data directory is created as a hidden directory.
    To see it, you must activate the property for displaying hidden files and folders for your operating system,
    if you have not already done so

* Get version info:
  ````shell
  hashguard --version
  ````

* Get general help:
  ````shell
  hashguard --help
  ````

* Get help on a specific command:
  ````shell
  hashguard download --help
  ````
  ````shell
  hashguard local --help
  ````

### Supported Hash Algorithms
* MD5
* SHA-1
* SHA2-256
* SHA2-512

## Notice
**_No colored console output under windows?_**
<br>
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
