# hashguard
[![Build Status](https://github.com/javaLux/hashguard/actions/workflows/ci.yml/badge.svg)](https://github.com/javaLux/hashguard/actions)
[![dependency status](https://deps.rs/repo/github/javaLux/hashguard/status.svg)](https://deps.rs/repo/github/javaLux/hashguard)
[![GitHub license](https://img.shields.io/github/license/javaLux/hashguard.svg)](https://github.com/javaLux/hashguard/blob/main/LICENSE)
[![crates.io](https://img.shields.io/crates/v/hashguard.svg)](https://crates.io/crates/hashguard)
![maintenance-status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

Command-Line tool for ensuring the integrity of files using hash sums

```
  ___ ___               .__      ________                       .___
 /   |   \_____    _____|  |__  /  _____/ __ _______ _______  __| _/
/    ~    \__  \  /  ___/  |  \/   \  ___|  |  \__  \\_  __ \/ __ | 
\    Y    // __ \_\___ \|   Y  \    \_\  \  |  // __ \|  | \/ /_/ | 
 \___|_  /(____  /____  >___|  /\______  /____/(____  /__|  \____ | 
       \/      \/     \/     \/        \/           \/           \/ 
```


# Overview
HashGuard is a lean and efficient command-line tool designed to simplify the process of downloading files from the internet and ensuring their integrity using hash sums. With HashGuard, you can confidently download files and verify their authenticity, providing an extra layer of security to your downloads. It is also possible to verify files and directories on the local system.

![Hashguard-Demo](../assets/hashguard-demo.gif?raw=true)

## Features
* ### Download-Command
  * Download a file and have a specific hash sum calculated depending on the selected hash algorithm
  * Or you can directly enter a known hash to compare it after the download.
    This allows you to check whether the file was changed during the download process
  * **_Notice_**
    * If you use the download command, please enclose the URL in double quotation marks.
      Because by enclosing the URL in double quotation marks, you tell the shell to treat the entire string as a single argument, even if it contains spaces or other special characters. This can prevent errors and unexpected behavior in your shell.
  
* ### Local-Command
  * Allows to hash local files, directories or any byte-buffer (furthermore you can also compare with a specific hash)
  * **Options**
    * _include-names_
        * Enables the inclusion of file and directory names for the calculation of the hash sum. This option only has an effect in conjunction with the ``--path`` option
    * _path_
      * Calculate a hash sum from a file/dir
    * _buffer_
      * Calculate a hash sum from any given byte buffer
      * What means byte buffer?
        * For example, you can calculate a hash sum from any text that is provided as a ``String``
        * As described in the download command, please enclose the text to be hashed in double quotation marks. This prevents unexpected behavior in your shell.
  * _Notice_
    * You can only use one option per call. So either ``path`` or ``buffer``


* **Hash Verification:** Verify the authenticity of downloaded or local files by comparing their hash with a specified hash. Any mismatched hash digits
will be highlighted (only if colored output is not disabled).
* **Support for Various Hash Algorithms:** HashGuard supports different hash algorithms, including SHA-2, SHA-3 family. The default Hash-Algorithm is SHA2-256.
* **Intuitive Command-Line Interface:** The simple and user-friendly CLI lets you easily calculate and compare hash sums.
* **Enable or Disable colored output**
* **Logging**
  * To enable logging, set one of the following log level options: [ `-l=debug|info`, `--logging=debug|info` ]
    * `debug` log level: write all available information to the log file
      * For example, if you use the `download` command, the whole http request and response is logged
    * `info` log level: write only necessary information to the log file (e.g. common application operations and error messages)

## Supported OS
* Linux
* MacOs
* Windows 10/11

## Installation
### Requirements
#### Installing Rust
To install Rust on your system, just go to the [official Rust website](https://www.rust-lang.org/tools/install), download and install the Rustup toolchain manager.

**Notice:**
Please refer to the installation instructions for your operating system. For certain operating systems, build tools need to be installed before you can use Rust.

HashGuard is available on [crates.io](https://crates.io/crates/hashguard) Rust community's crate registry.
So you can easily install it as binary on your local system.
Use the follow command:
```
cargo install hashguard
```
### Using Natively Compiled Binaries
If you don't have Rust installed or prefer not to build the project yourself, you can use the precompiled binaries provided in the [Releases](https://github.com/javaLux/hashguard/releases) section. Download the appropriate binary for your operating system and architecture, and you're good to go!

- [Download the latest release](https://github.com/javaLux/hashguard/releases/latest)

## Build the project

**_To build this project from scratch follow these steps:_**

* Clone this repository
* Open a terminal
* Navigate to the root directory of the project
* Run the following command
```
cargo build --release
```
* The compiled binary will be available at `./target/release/`

## How to use
### General Syntax
* ``hashguard [OPTIONS] <COMMAND>``

### Command specific syntax
* ``hashguard [OPTIONS] download [OPTIONS] <URL> [HASH]``
* ``hashguard [OPTIONS] local [OPTIONS] [HASH]``

### Passing a Hash
If you want to specify a hash for comparison, you can pass it as usual as a string with valid hexadecimal digits.
* For example:
  ````shell
      SHA2-256 Hash
      "9e2a73027d72a28e5cb05cf9e87e71d5f5850d047a8b163f92f2189e5e8f42ac"
  ````

It is also possible to add a **_prefix_** to the hash to define the hash algorithm to be used.

* For example:
  ````shell
      SHA2-256 Hash with prefix
      "sha256:9e2a73027d72a28e5cb05cf9e87e71d5f5850d047a8b163f92f2189e5e8f42ac"
  ````
<br>

**Supported prefixes:**

| Prefix                 |   Associated algorithm   |
|------------------------|--------------------------|
| `sha224`, `sha2-224`, `sha2_224` | SHA2-224 |
| `sha256`, `sha2-256`, `sha2_256` | SHA2-256 *(Default)* |
| `sha384`, `sha2-384`, `sha2_384` | SHA2-384 |
| `sha512`, `sha2-512`, `sha2_512` | SHA2-512 |
| `sha3-224`, `sha3_224` | SHA3-224 |
| `sha3-256`, `sha3_256` | SHA3-256 |
| `sha3-384`, `sha3_384` | SHA3-384 |
| `sha3-512`, `sha3_512` | SHA3-512 |

> **Note:**  
> If neither a prefix nor the option ``[-a, --algorithm]`` is specified, **SHA2-256** is automatically used as the default algorithm.
> If a hash is passed with a prefix, the ``[-a, --algorithm]`` option is ignored by default.


### Usage Examples

**Common**
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

**Download-Command**
  * Download a file and verify it with a hash sum by using the default hash algorithm SHA2-256:
    ````shell
    hashguard download "https://example.com/file.zip" a1b2c3d4e5f6
    ````

  * Download a file and calculate a hash sum with a specific hash algorithm:
    ````shell
    hashguard download "https://example.com/file.zip" -a sha2-512
    ````
  
  * Download a file and use a prefixed hash to specify the algorithm:
    ````shell
    hashguard download "https://example.com/file.zip" sha224:a1b2c3d4e5f6
    ````

  * Use a specific output directory for the downloaded file:
    ````shell
    hashguard download "https://example.com/image.jpg" a1b2c3d4e5f6 -o /path/to/output_directory
    ````

  * Use the --rename option to rename the file to be downloaded:
    ````shell
    hashguard download "https://example.com/image.jpg" a1b2c3d4e5f6 -r "my_fancy_new_file.jpg"
    ````
  * Disable colored output:
    ````shell
    hashguard -c download "https://example.com/file.zip"
    ````

**Local-Command**
  * Verify a local file with a hash sum using SHA-3:
    ````shell
    hashguard local -p /path/to/local_file.txt a1b2c3d4e5f6 -a sha3-256
    ````

  * Calculate a hash sum from a given ``String``:
    ````shell
    hashguard local -b "Hello my eager young Padawan"
    ````

  * Calculate a hash sum from a local directory with the default hash algorithm:
    ````shell
    hashguard local -p /path/to/test_dir
    ````
  
  * Calculate a hash sum from a local file and save the calculated hash to a file:
    ````shell
    hashguard -s local -p /path/to/local_file.txt
    ````
    * The file containing the calculated hash following by the input source (e.g. Path or the byte buffer)
    * You find the file in the application data directory.

**Use Logging**
  * Enable `debug` log level:
    ````shell
    hashguard -l debug download "https://example.com/file.zip" a1b2c3d4e5f6
    ````
  * Enable `info` log level
    ````shell
    hashguard -l info local -p /path/to/local_file.txt
    ````
  * All logs are written to a log file stored in the application's data directory.
  * You can find out the application data directory with the [ `-V`, `--version` ] command

### Supported Hash Algorithms
* SHA2-224
* SHA2-256
* SHA2-384
* SHA2-512
* SHA3-224
* SHA3-256
* SHA3-384
* SHA3-512

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

## Contributing
Contributions are very welcome! Whether it's a bug fix, improvement, or new feature — your help is appreciated. To contribute, follow these simple steps:

1. **Fork** this repository.
2. **Create a new branch** for your changes.
3. Make your changes, ensuring they are clean and well-documented.
4. **Commit** your changes with a meaningful message.
5. **Push** your branch to your fork.
6. Open a pull request from your branch to the main repository.

Please ensure your code passes any tests and follows the existing style. If you're not sure where to start, feel free to open an issue and ask!

## License
HashGuard is released under the MIT License.

## Disclaimer
While HashGuard aims to provide reliable file verification, it is essential to exercise caution when downloading files from the internet or using local files for verification. Always ensure that you trust the source and the provided hash sum before proceeding.
