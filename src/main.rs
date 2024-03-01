// make own custom modules available
mod cli;
mod color_templates;
mod commands;
mod download;
mod filename_handling;
mod os_specifics;
mod panic_handling;
mod tests;
mod utils;
mod verify;

use clap::Parser;
use color_eyre::eyre::Result;
use color_templates::*;

fn main() -> Result<()> {
    // Parse the given CLI-Arguments
    let args = cli::Cli::parse();

    utils::initialize_logging(args.log_level)?;
    utils::initialize_panic_hook(args.log_level)?;

    // get the underlying os type
    let os_type = os_specifics::get_os();

    match os_type {
        Some(os) => {
            // check which command is given (download or local)
            match args.command {
                cli::Commands::Download(args) => {
                    if let Err(cmd_err_report) = commands::handle_download_cmd(args, os) {
                        log::error!(
                            "An application error occurred - Details: {:?}",
                            cmd_err_report.root_cause()
                        );
                        return Err(cmd_err_report);
                    }
                }
                cli::Commands::Local(args) => {
                    if let Err(cmd_err_report) = commands::handle_local_cmd(args) {
                        log::error!(
                            "An application error occurred - Details: {:?}",
                            cmd_err_report.root_cause()
                        );
                        return Err(cmd_err_report);
                    }
                }
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
