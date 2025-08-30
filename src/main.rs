mod app;
mod cli;
mod color_templates;
mod command_handling;
mod download;
mod filename_handling;
mod hasher;
mod local;
mod os_specifics;
mod panic_handling;
mod utils;

use crate::color_templates::{ERROR_TEMPLATE_NO_BG_COLOR, WARN_TEMPLATE_NO_BG_COLOR};
use anyhow::Result;
use clap::Parser;

fn run() -> Result<()> {
    // get the underlying os type
    let os_type = os_specifics::get_os();

    match os_type {
        Some(os) => {
            // Parse the given CLI-Arguments
            let args = cli::Cli::parse();

            app::initialize_logging(args.logging)?;
            panic_handling::initialize_panic_hook()?;
            app::set_ctrl_c_handler()?;
            // check which command is given (download or local)
            match args.command {
                cli::Command::Download(args) => command_handling::handle_download_cmd(args, os)?,
                cli::Command::Local(args) => command_handling::handle_local_cmd(args)?,
            }
        }
        // Only Linux, MacOs and Windows are supported
        None => {
            eprintln!(
                "{} - Supported OS: {}",
                WARN_TEMPLATE_NO_BG_COLOR.output(
                    "Could not execute the program, the current Operating-System is unsupported."
                ),
                format_args!(
                    "[{:?}, {:?}, {:?}]",
                    os_specifics::OS::Linux,
                    os_specifics::OS::MacOs,
                    os_specifics::OS::Windows
                ),
            );
        }
    }

    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{}\n{}\n", ERROR_TEMPLATE_NO_BG_COLOR.output("error:"), e);
    }
}
