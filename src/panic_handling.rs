use std::{backtrace::Backtrace, io::Write, panic::PanicInfo, path::Path};

use crate::utils;

/// Environment variables Cargo sets for crates.
/// Cargo exposes these environment variables to your crate when it is compiled.
#[derive(Debug)]
pub struct CargoMetadata {
    /// The crate name
    pub crate_name: String,
    /// The crate version
    pub crate_version: String,
    /// The list of authors of the crate
    pub crate_authors: String,
    /// The URL of the crate's website
    pub crate_homepage: String,
    /// The repository from the manifest of your package
    pub crate_repository: String,
    /// The running operating system
    pub operating_system: String,
}

impl Default for CargoMetadata {
    /// Collect the available project metadata provided by Cargo and construct a new instance of [CargoMetadata]
    fn default() -> Self {
        let crate_name = {
            let name = env!("CARGO_PKG_NAME").trim().to_string();
            if !name.is_empty() {
                name
            } else {
                "Unknown".to_string()
            }
        };
        let crate_version = {
            let version = env!("CARGO_PKG_VERSION").trim().to_string();
            if !version.is_empty() {
                version
            } else {
                "Unknown".to_string()
            }
        };
        let crate_authors = {
            let authors = env!("CARGO_PKG_AUTHORS").trim().to_string();
            if !authors.is_empty() {
                authors.replace(':', ", ")
            } else {
                "Unknown".to_string()
            }
        };
        let crate_homepage = {
            let homepage = env!("CARGO_PKG_HOMEPAGE").trim().to_string();
            if !homepage.is_empty() {
                homepage
            } else {
                "Unknown".to_string()
            }
        };
        let crate_repository = {
            let repository = env!("CARGO_PKG_REPOSITORY").trim().to_string();
            if !repository.is_empty() {
                repository
            } else {
                "Unknown".to_string()
            }
        };

        let operating_system: String = os_info::get().to_string();
        Self {
            crate_name,
            crate_version,
            crate_authors,
            crate_homepage,
            crate_repository,
            operating_system,
        }
    }
}

impl std::fmt::Display for CargoMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pretty_print = format!(
            "crate_name      : {}\ncrate_version   : {}\ncrate_authors   : {}\ncrate_homepage  : {}\ncrate_repository: {}\noperating_system: {}\n",
            self.crate_name,
            self.crate_version,
            self.crate_authors,
            self.crate_homepage,
            self.crate_repository,
            self.operating_system,
        );

        write!(f, "{}", pretty_print)
    }
}

#[derive(Debug)]
pub struct PanicReport<'a> {
    panic_info: &'a PanicInfo<'a>,
    backtrace: Backtrace,
}

/// A human readable crash report
#[derive(Debug, Default)]
struct HumanReadableReport {
    cargo_metadata: CargoMetadata,
    explanation: String,
    cause: String,
    backtrace: String,
}

impl HumanReadableReport {
    fn explanation(mut self, explanation: String) -> Self {
        self.explanation = explanation;
        self
    }
    fn cause(mut self, cause: String) -> Self {
        self.cause = cause;
        self
    }
    fn backtrace(mut self, backtrace: String) -> Self {
        self.backtrace = backtrace;
        self
    }
    fn serialize(&self) -> String {
        format!(
            "{}\nexplanation: {}\ncause: {}\n{}",
            self.cargo_metadata, self.explanation, self.cause, self.backtrace
        )
    }
}

impl<'a> PanicReport<'a> {
    /// Constructs a new instance of [`PanicReport`].
    pub fn new(panic_info: &'a PanicInfo, backtrace: Backtrace) -> Self {
        Self {
            panic_info,
            backtrace,
        }
    }

    ///  Try to create the Log-File and write the report
    pub fn write_report(&self, file_path: &Path) -> color_eyre::eyre::Result<()> {
        let report = self.build_human_readable_report();

        match std::fs::File::create(file_path) {
            Ok(mut log_file) => match log_file.write_all(report.as_bytes()) {
                Ok(_) => {
                    eprintln!(
                        "\n- A crash report file was generated: '{}' \
                        \n- Submit an issue or email with the subject of '{} Crash Report' \
                            and include the report as an attachment. \
                        \n- The project repository and much more can be found in the crash report file.",
                        utils::get_absolute_path(file_path),
                        env!("CARGO_PKG_NAME")
                    );
                }
                Err(io_err) => {
                    let err_msg = format!(
                        "Unable to write crash report to log file: {} - Details: {:?}",
                        utils::get_absolute_path(file_path),
                        io_err
                    );
                    return Err(color_eyre::eyre::eyre!(err_msg));
                }
            },
            Err(io_err) => {
                let err_msg = format!(
                    "Unable to create log file: {} - Details: {:?}",
                    utils::get_absolute_path(file_path),
                    io_err
                );
                return Err(color_eyre::eyre::eyre!(err_msg));
            }
        }
        Ok(())
    }

    fn build_human_readable_report(&self) -> String {
        #[cfg(feature = "nightly")]
        let message = panic_info.message().map(|m| format!("{}", m));

        #[cfg(not(feature = "nightly"))]
        let message = match (
            self.panic_info.payload().downcast_ref::<&str>(),
            self.panic_info.payload().downcast_ref::<String>(),
        ) {
            (Some(s), _) => Some(s.to_string()),
            (_, Some(s)) => Some(s.to_string()),
            (None, None) => None,
        };

        let cause = match message {
            Some(m) => m,
            None => "Unknown".into(),
        };

        let panic_location = match self.panic_info.location() {
            Some(location) => format!(
                "Panic occurred in file '{}' at line {}",
                location.file(),
                location.line()
            ),
            None => "Panic location unknown".to_string(),
        };

        let backtrace = format!("{:#?}", self.backtrace);

        HumanReadableReport::default()
            .explanation(panic_location)
            .cause(cause)
            .backtrace(backtrace)
            .serialize()
    }
}
