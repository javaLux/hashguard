# HashGuard CLI Changelog

All notable changes to this project will be documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## [1.0.0] - 2023-08-01
### Added
- Initial release of HashGuard CLI Tool.
- Basic functionality to download and verify files using hash sums.

### Changed
- Updated README with usage instructions.

## [1.0.1] - 2023-08-01
### Changed
- Improve ``README.md``

## [1.0.2] - 2023-08-02
### Changed
- Update dependencies ``Cargo.lock``

## [1.0.3] - 2023-08-09
### Changed
- Update dependencies ``Cargo.lock``
- Improved error handling and messages.

## [1.0.4] - 2023-08-15
### Changed
- Improve user specific output
- Improve the general program logic
- Update [chksum](https://crates.io/crates/chksum/0.2.0) library to ``v0.2.0``
- Update dependencies ``Cargo.lock``
- Improve the usage of the ``chksum`` library
- A big thank you to the following contributors without whom the above changes would not have been possible:
- [@ventaquil](https://github.com/ventaquil)

## [1.0.5] - 2023-08-22
### Changed
- Update dependencies ``Cargo.lock`` + ``Cargo.toml``
- fix vulnerabilities -> rustls-webpki: CPU denial of service in certificate path building

## [1.0.6] - 2023-10-18
### Changed
- Update dependencies ``Cargo.lock`` + ``Cargo.toml``
- fix vulnerabilities

## [1.0.7] - 2023-12-10
### Changed
- Update dependencies ``Cargo.lock`` + ``Cargo.toml``
- Refactoring
- Improvement of the code base
- Improvement of the user messages
- Improvement of error handling - use [color-eyre](https://crates.io/crates/color-eyre) as error report handler
- Add a new CLI-Option [-r, --rename] for the ``download`` command -> you can now specify a new file name for the file to be downloaded

## [1.0.8] - 2023-12-12
### Added
- dependency [path-absolutize](https://crates.io/crates/path-absolutize)

### Fixed
- Fixed bug that the absolute path was not displayed correctly under Windows

### Changed
- improvement of error messages