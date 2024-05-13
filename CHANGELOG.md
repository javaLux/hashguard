# HashGuard CLI Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Released]

## [2.0.3] - 2024-05-13

### Fixed
- Bug fix for extracting filename from a specified URL if the ``Content-Disposition`` header not present
  - In the case that the specified URL contains query parameters (URL parameters) the filename could not extract correctly
  - This bug is fixed in this version

### Changed
- Refactoring
- Update dependencies
<br>

## [2.0.2] - 2024-05-02
### Added
- Add a Signal handler for ``Ctrl_C``

### Changed
- Improve user error messages
- Improve debug log messages
- Refactoring
- Update dependencies
<br>

## [2.0.1] - 2024-04-23
### Security
- Bump rustls from 0.22.2 to 0.22.4
  - ``rustls::ConnectionCommon::complete_io`` could fall into an infinite loop based on network input.

### Changed
- Update dependencies
<br>

## [2.0.0] - 2024-03-02
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

### Changed
- Update dependencies
- Refactoring
<br>

## [1.0.8] - 2023-12-12
### Added

### Fixed
- Bug fix that the absolute path was not displayed correctly under Windows
  - using dependency [path-absolutize](https://crates.io/crates/path-absolutize)

### Changed
- improvement of error messages
<br>

## [1.0.7] - 2023-12-10
### Changed
- Update dependencies
- Refactoring
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

















