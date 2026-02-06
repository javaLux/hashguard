mod app;
mod cli;
mod command_handling;
mod download;
mod filename_handling;
mod hasher;
mod local;
mod os_specifics;
mod panic_handling;
mod term_output;
mod utils;

use anyhow::Result;
use clap::Parser;
use std::io::Write;
use termcolor::{Color, ColorSpec, WriteColor};

use crate::{
    app::{APP_NAME, run},
    cli::Cli,
};

fn main() -> Result<()> {
    // Parse the given CLI-Arguments
    let args = Cli::parse();
    let no_color = args.no_color;

    if let Some(os) = os_specifics::get_os() {
        if let Err(e) = run(args, os) {
            let mut stdout = term_output::get_stdout(no_color);
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;

            writeln!(stdout, "\nAn error occurred while running {}:", APP_NAME)?;
            term_output::reset_color(&mut stdout)?;

            writeln!(stdout, "{e}\n")?;
        }
    } else {
        let mut stdout = term_output::get_stdout(no_color);
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;

        writeln!(
            stdout,
            "Could not execute {}, the current Operating-System is unsupported",
            APP_NAME
        )?;

        term_output::reset_color(&mut stdout)?;
        writeln!(
            stdout,
            "Supported OS: {}",
            format_args!(
                "[{:?}, {:?}, {:?}]",
                os_specifics::OS::Linux,
                os_specifics::OS::MacOs,
                os_specifics::OS::Windows
            )
        )?;
    }
    Ok(())
}
