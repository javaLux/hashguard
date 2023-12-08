// make own custom modules available
mod cli;
mod color_templates;
mod commands;
mod download;
mod filename_handling;
mod os_specifics;
mod util;
mod verify;

use clap::Parser;
use color_eyre::eyre::Result;
use color_templates::*;

fn main() -> Result<()> {
    // set up error and panic handler
    color_eyre::install()?;
    // Parse the given CLI-Arguments
    let cli_args = cli::Cli::parse();

    // After parsing the cli args, check if the underlying OS is supported
    let os_type = os_specifics::get_os();

    match os_type {
        Some(os) => {
            // check which command is given (download or local)
            match cli_args.command {
                cli::Commands::Download(download_args) => {
                    let used_alg = download_args.algorithm;
                    let command_result = commands::perform_download_command(download_args, os)?;
                    // print the result
                    util::generate_output(command_result.is_file_modified, used_alg);

                    // Finally print the file location of the downloaded file
                    println!(
                        "{}: {}",
                        WARN_TEMPLATE_NO_BG_COLOR.output("File location"),
                        command_result.file_destination.display()
                    );
                }
                cli::Commands::Local(local_args) => {
                    let used_alg = local_args.algorithm;
                    let is_file_modified = commands::is_local_file_modified(local_args)?;
                    util::generate_output(is_file_modified, used_alg);
                }
            }
        }
        // Only Linux, MacOsX and Windows are supported
        None => {
            println!(
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
