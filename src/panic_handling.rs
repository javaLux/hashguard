use crate::{
    app::{data_dir, set_rust_backtrace, APP_NAME},
    color_templates::WARN_TEMPLATE_NO_BG_COLOR,
    utils,
};
use anyhow::{Context, Result};
use std::{
    backtrace::Backtrace,
    io::Write,
    panic::PanicHookInfo,
    path::{Path, PathBuf},
};

/// Define a custom panic hook to handle a application crash.
/// Try to reset the terminal properties in case of the application panicked (crashed).
/// This way, you won't have your terminal messed up if an unexpected error happens.
pub fn initialize_panic_hook() -> Result<()> {
    // set the RUST_BACKTRACE environment variable to 1
    set_rust_backtrace();
    // set the custom panic hook handler
    std::panic::set_hook(Box::new(move |panic_info| {
        // write the Crash-Report file
        let crash_report_file = crash_report_file();

        let backtrace = std::backtrace::Backtrace::capture();
        let panic_report = PanicReport::new(panic_info, backtrace);
        if let Err(err) = panic_report.write_report_and_print_msg(&crash_report_file) {
            log::error!("{err}");
            eprintln!("{err}")
        }

        std::process::exit(1);
    }));
    Ok(())
}

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

        write!(f, "{pretty_print}")
    }
}

#[derive(Debug)]
pub struct PanicReport<'a> {
    panic_info: &'a PanicHookInfo<'a>,
    backtrace: Backtrace,
}

/// A human readable crash report
#[derive(Debug, Default)]
struct HumanReadableReport {
    cargo_metadata: CargoMetadata,
    explanation: String,
    cause: String,
    backtrace: String,
    thread_name: String,
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
    fn thread_name(mut self, thread_name: &str) -> Self {
        self.thread_name = thread_name.to_string();
        self
    }

    fn serialize(&self) -> String {
        format!(
            "{}\nexplanation: {}\ncause      : {}\nthread     : {}\n\n{}",
            self.cargo_metadata, self.explanation, self.cause, self.thread_name, self.backtrace
        )
    }
}

impl<'a> PanicReport<'a> {
    /// Constructs a new instance of [`PanicReport`].
    pub fn new(panic_info: &'a PanicHookInfo, backtrace: Backtrace) -> Self {
        Self {
            panic_info,
            backtrace,
        }
    }

    ///  Try to create the Log-File and write the report
    pub fn write_report_and_print_msg(&self, p: &Path) -> Result<()> {
        let report = self.build_human_readable_report();

        let mut crash_report = std::fs::File::create(p).with_context(|| {
            format!(
                "Failed to create Crash-Report file: {}",
                utils::absolute_path_as_string(p)
            )
        })?;

        crash_report.write_all(report.as_bytes()).with_context(|| {
            format!(
                "Failed to write crash report to file: {}",
                utils::absolute_path_as_string(p),
            )
        })?;

        let path_to_crash_report = utils::absolute_path_as_string(p);

        println!("\n{}", WARN_TEMPLATE_NO_BG_COLOR.output("The application panicked (crashed). Please see the Crash-Report file for more information"));
        println!(
            "\n- A crash report file was generated: '{}' \
            \n- Submit an issue to: '{}/issues' with the subject of '{} Crash Report' \
                and include the report as an attachment. \
            \n- Thank you for your help!",
            path_to_crash_report,
            env!("CARGO_PKG_REPOSITORY"),
            APP_NAME
        );
        Ok(())
    }

    fn build_human_readable_report(&self) -> String {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("<unnamed>");

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
                "Panic occurred in file '{}' at line '{}'",
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
            .thread_name(thread_name)
            .serialize()
    }
}

fn crash_report_file() -> PathBuf {
    let crash_report_file_name = format!(
        "{}-Crash-Report_{}.log",
        APP_NAME,
        chrono::Local::now().format("%Y-%m-%dT%H_%M_%S")
    );
    data_dir().join(crash_report_file_name)
}
