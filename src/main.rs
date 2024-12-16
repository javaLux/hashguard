mod app;
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
    // get the underlying os type
    let os_type = os_specifics::get_os();

    match os_type {
        Some(os) => {
            // Parse the given CLI-Arguments
            let args = cli::Cli::parse();

            app::initialize_logging(args.log_level)?;
            app::initialize_panic_hook(args.log_level)?;
            app::set_ctrl_c_handler()?;
            // check which command is given (download or local)
            match args.command {
                cli::Commands::Download(args) => commands::handle_download_cmd(args, os)?,
                cli::Commands::Local(args) => commands::handle_local_cmd(args)?,
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
