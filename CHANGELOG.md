# HashGuard CLI Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Released]

## [4.0.1] - 2025-05-08
- Update dependencies
- Refactoring
- Improved error handling

### Added
- **Local-Command**
  - Adding the processed bytes to the ProgressBar spinner during the hash sum calculation
<br>


## [4.0.0] - 2025-05-04
- Update dependencies
- The calculation of the hash sum when downloading a file has been made more efficient. Now the hash sum is calculated directly during the download, which  reduces CPU and memory usage. As only small blocks of data are used for the calculation. The same applies to local files and directories.

### Changed
- Insecure hash algorithms have been removed (MD5, SHA1)

### Added
- ``SHA3`` hash algorithm is now supported
- ``-s, --save`` flag, to be able to store the calculated hash sum in a file, stored in the app data directory
- **Local-Command**
  - It is now possible to include file and directory names in the calculation of the hash sum by using the ``-i, --include-names`` flag
<br>


## [3.1.1] - 2025-03-23
- Update dependencies
<br>


## [3.1.0] - 2025-02-28
- Update dependencies
- Improved panic handling to enhance stability
- Enhanced error messages for better debugging
- Improved CLI command descriptions for clarity
- Refactoring

### Added
- pre compiled binaries for Linux, MacOs and Windows
  - provided builds for `x86_64` and `aarch_64(ARM64)`

### Changed
- The data directory for log and crash report files has been moved to the user data directory, which varies based on the operating system.
  - It is no longer created as a hidden directory.
- Updated logging mechanism:
  - Instead of creating a new log file on every app start, logs are now written to a single file until it reaches **1MB**.
  - Once the file reaches this limit, it is renamed with the `.old` suffix, and a new log file is created.
  - This prevents excessive log file creation and improves log management.
- Switched from `color-eyre` to `anyhow` for error handling.
- Removed stack trace display when `logging=debug`.
<br>


## [3.0.5] - 2025-01-12
- Update dependencies

### Fixed
- with the `download` command, the reading of the file name from the server response was not implemented correctly
- this bug has been fixed in this version
<br>


## [3.0.4] - 2025-01-01
- Refactoring
- Update dependencies
- improve user error messages

### Fixed
- report handler crashed(panicked) on windows if `RUST_BACKTRACE` set to full
- `download` command
  - parsing `Content-Range` header was incorrect
  - if the header specifies an unknown content size (e.g. Content-Range: bytes 0-1023/*), this was not taken into account
  - now a streamed download is initiated in this case

### Added
- `info` log level to write only necessary information to the log file
- `download` command
  - set connection time out to 25 sec
  - display user info during the connection establishment
  - improve logging behavior by using the debug log level
    - in case of an successfully request, the response headers will be logged
    - in case of an failed request, the response headers and the response body will be logged
<br>


## [3.0.3] - 2024-12-16
- Performance optimizations
- Refactoring
- Update dependencies

### Changed
- Validation improvements of download command options
  - filename validation => option `[-r, --rename]`
  - output target validation => option `[-o, --output]`
<br>


## [3.0.2] - 2024-12-11

- Optimized binary size => disable dependency default features
- Bug fixing when comparing hash sums
- Update dependencies

### Changed
- Improve error messages

### Added
- Validation of a given hash sum, only a valid hexadecimal digit is accepted for the comparison
<br>


## [3.0.1] - 2024-12-08

### Changed
- ``local`` subcommand
  - option `--file` was renamed to `--path`
  - now you use a path pointing to a file or a directory and calculate a hash sum
- Refactoring
<br>


## [3.0.0] - 2024-11-08

- Update dependencies

### Changed
- the ``local`` subcommand now provides two options [`--file`, `--buffer`], the file path no longer needs to be specified via a separate argument
  - `--file` calculate a hash sum from a local file
  - `--buffer` calculate a hash sum from any given byte-buffer (e.g. String)
- Refactoring

### Added
- Now the following hash algorithms are available:
  - SHA2-224
  - SHA2-384
<br>


## [2.0.5] - 2024-08-22

- Refactoring
- Update dependencies

### Changed
- Improve user and log error messages
- Now ``chunked`` file downloads are supported
<br>


## [2.0.4] - 2024-06-11

- Update dependencies

### Changed
- Improvement of the URL validation for the ``download`` command, e.g. only the protocols ``http`` and ``https`` are supported
<br>


## [2.0.3] - 2024-05-13

- Refactoring
- Update dependencies

### Fixed
- Bug fix for extracting filename from a specified URL if the ``Content-Disposition`` header not present
  - In the case that the specified URL contains query parameters (URL parameters) the filename could not extract correctly
  - This bug is fixed in this version
<br>


## [2.0.2] - 2024-05-02

- Refactoring
- Update dependencies

### Added
- Add a Signal handler for ``Ctrl_C``

### Changed
- Improve user error messages
- Improve debug log messages
<br>


## [2.0.1] - 2024-04-23

- Update dependencies

### Security
- Bump rustls from 0.22.2 to 0.22.4
  - ``rustls::ConnectionCommon::complete_io`` could fall into an infinite loop based on network input.
<br>


## [2.0.0] - 2024-03-02

- Update dependencies
- Refactoring

### Added
- Implemented a panic handler to create a `Crash-Report.log` file in case of application crashes. This file contains information about the OS metadata and the crash, aiding in debugging and issue resolution.

- Introduced a log level configuration feature. Users can now set the log level to either "info" or "debug" during application runtime.
  - When the log level is set to "debug," a log file is generated in the user's home directory. This file contains all log output generated during the application run.
  - use [-l, --log-level] flag the set the application log level

- **It is now possible to calculate a hash sum without having to specify a comparison hash sum**

### Fixed
- Bug fix that sometimes the correct file name could not be extracted when executing the ``Download`` command
  - Correctly extract filenames from redirect URLs
  - using the ``Content-Disposition`` HTTP-Header to extract filename
<br>


## [1.0.8] - 2023-12-12

### Fixed
- Bug fix that the absolute path was not displayed correctly under Windows
  - using dependency [path-absolutize](https://crates.io/crates/path-absolutize)

### Changed
- improvement of error messages
<br>


## [1.0.7] - 2023-12-10

- Update dependencies
- Refactoring

### Changed
- Improvement of the code base
- Improvement of the user messages
- Improvement of error handling - use [color-eyre](https://crates.io/crates/color-eyre) as error report handler
- Add a new CLI-Option [-r, --rename] for the ``download`` command -> you can now specify a new file name for the file to be downloaded
<br>


## [Unreleased]

## [1.0.6] - 2023-10-18
### Changed
- Update dependencies
- fix vulnerabilities
<br>

## [1.0.5] - 2023-08-22
### Changed
- Update dependencies
- fix vulnerabilities -> rustls-webpki: CPU denial of service in certificate path building
<br>

## [1.0.4] - 2023-08-15
### Changed
- Improve user specific output
- Improve the general program logic
- Update [chksum](https://crates.io/crates/chksum/0.2.0) library to ``v0.2.0``
- Update dependencies ``Cargo.lock``
- Improve the usage of the ``chksum`` library
- A big thank you to the following contributors without whom the above changes would not have been possible:
- [@ventaquil](https://github.com/ventaquil)
<br>

## [1.0.3] - 2023-08-09
### Changed
- Update dependencies
- Improved error handling and messages.
<br>

## [1.0.2] - 2023-08-02
### Changed
- Update dependencies
<br>

## [1.0.1] - 2023-08-01
### Changed
- Improve ``README.md``
<br>

## [1.0.0] - 2023-08-01
### Added
- Initial release of HashGuard CLI Tool.
- Basic functionality to download and verify files using hash sums.

### Changed
- Updated README with usage instructions.

















