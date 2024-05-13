use std::{
    cmp::min,
    error::Error,
    fmt,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use color_eyre::eyre::Result;

use crate::{
    app, color_templates::WARN_TEMPLATE_NO_BG_COLOR, filename_handling, os_specifics::OS, utils,
};

use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone)]
enum DownloadError {
    DownloadFailed(String),
    DownloadInterrupted,
}

impl Error for DownloadError {}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadError::DownloadFailed(err) => {
                write!(f, "Download failed - {}", err)
            }
            DownloadError::DownloadInterrupted => {
                write!(f, "Download interrupted")
            }
        }
    }
}

#[derive(Debug)]
pub struct DownloadProperties {
    pub url: String,
    pub output_target: PathBuf,
    pub default_file_name: Option<String>,
    pub os_type: OS,
}

/// Executes the file download for the specified URL and returns the path where the file was saved
/// * Make a HTTP-GET request
/// * Check the server response for errors
/// * Verify the response for the required HTTP headers
/// * Starts a progress bar to display the download progress
/// * Write all bytes from the HTTP response body to a file in 8KiB blocks
pub fn make_download(download_properties: DownloadProperties) -> Result<PathBuf> {
    let http_agent = ureq::builder()
        .timeout_connect(Duration::from_secs(8))
        .build();

    // make a HTTP-Get request to the server
    let response = match http_agent.get(&download_properties.url).call() {
        Ok(response) => response,
        Err(req_err) => {
            // if the download failed - fetch the concrete error type
            match req_err {
                ureq::Error::Status(code, response_err) => {
                    let err_description =
                        format!("HTTP-Status: {}, Response: {:?}", code, response_err);
                    return Err(color_eyre::eyre::eyre!(DownloadError::DownloadFailed(
                        err_description
                    )
                    .to_string()));
                }
                ureq::Error::Transport(transport_err) => {
                    let err_description =
                        format!("due to HTTP transport error: {:?}", transport_err);
                    return Err(color_eyre::eyre::eyre!(DownloadError::DownloadFailed(
                        err_description
                    )
                    .to_string()));
                }
            }
        }
    };

    log::debug!("{:?}", response);

    // get the Content-Length header
    let content_length = response
        .header(utils::CONTENT_LENGTH_HEADER)
        .unwrap_or_default();

    if content_length.is_empty() {
        let err_description = format!(
            "The server did not send a {} header",
            utils::CONTENT_LENGTH_HEADER
        );
        return Err(color_eyre::eyre::eyre!(DownloadError::DownloadFailed(
            err_description
        )));
    }

    // IMPORTANT: use the url from the response object, because in case of an redirect the
    // url can differ from the request url when the http client has follows redirects.
    let url = response.get_url();

    // get the Content-Disposition header
    let content_disposition = response
        .header(utils::CONTENT_DISPOSITION_HEADER)
        .unwrap_or_default();

    let extract_result = match download_properties.default_file_name {
        Some(default_file_name) => Some(default_file_name),
        None => {
            // if the user has not specified a default filename via the --rename option
            // -> try to extract the filename from the server response
            utils::extract_file_name(url, content_disposition, &download_properties.os_type)
        }
    };

    // check if a filename was found, if not the user have to enter a valid filename
    let filename = match extract_result {
        Some(filename) => filename,
        None => {
            println!(
                "{}",
                WARN_TEMPLATE_NO_BG_COLOR
                    .output("Could not extract a filename from server response")
            );
            println!("Please enter a name for the file to be downloaded");
            filename_handling::enter_and_verify_file_name(&download_properties.os_type)?
        }
    };

    // build the final path under which the file is saved
    let final_file_path = download_properties.output_target.join(filename.clone());

    // Try to parse the value from the Content-Length header as integer
    let content_length = match content_length.parse::<usize>() {
        Ok(content_length) => content_length,
        Err(parse_err) => {
            let err_description = format!(
                "The server has sent an invalid value for the {} header - Details: {:?}",
                utils::CONTENT_LENGTH_HEADER,
                parse_err
            );
            return Err(color_eyre::eyre::eyre!(DownloadError::DownloadFailed(
                err_description
            )
            .to_string()));
        }
    };

    if content_length < 1 {
        let err_description = format!(
            "The server sent a {} header of zero",
            utils::CONTENT_LENGTH_HEADER
        );
        Err(color_eyre::eyre::eyre!(DownloadError::DownloadFailed(
            err_description
        )))
    } else {
        // buffer size 8KiB - 8192 bytes
        let mut buffer = [0; 8192];

        // create the file to write in
        let file = File::create(&final_file_path)?;

        let mut writer = BufWriter::new(file);

        // capture response reader to read bytes from HTTP body
        let mut body = response.into_reader();

        log::debug!("Downloading file with the name: {}", filename,);
        log::debug!(
            "Output target: {}",
            utils::get_absolute_path(&download_properties.output_target)
        );

        let mut downloaded_bytes: usize = 0;

        // build the progress bar
        let progress_bar = ProgressBar::new(content_length as u64);
        progress_bar.set_style(
            ProgressStyle::with_template(
                "[{msg}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
            )
            .unwrap_or(ProgressStyle::default_bar())
            .progress_chars("#>-"),
        );

        progress_bar.set_message("Download in progress");

        // Start measuring time for the download
        let start = Instant::now();

        let download_result: Result<(), DownloadError> = loop {
            // check the app state -> if ctrl_c was pressed, abort the download
            if !app::APP_SHOULD_RUN.load(Ordering::SeqCst) {
                break Err(DownloadError::DownloadInterrupted);
            }

            // try to read from the response body
            let read_result = body.read(&mut buffer);

            match read_result {
                Ok(bytes_read) => {
                    // try to write read bytes into the BufWriter
                    let write_result = writer.write_all(&buffer[..bytes_read]);

                    if let Err(write_err) = write_result {
                        let err_description =
                            format!("Unable to write data from server response into file '{}' - Details: {:?}", utils::get_absolute_path(&final_file_path), write_err);
                        break Err(DownloadError::DownloadFailed(err_description));
                    }

                    // capture the successfully downloaded bytes
                    downloaded_bytes += bytes_read;

                    // get the right value for the progress bar
                    let pb_value = min(downloaded_bytes, content_length);

                    progress_bar.set_position(pb_value as u64);

                    // Break the loop if there are no more bytes to read
                    if bytes_read == 0 {
                        break Ok(());
                    }
                }
                Err(body_access_err) => {
                    let err_description = format!(
                        "Could not read from the server response - Details: {:?}",
                        body_access_err
                    );
                    break Err(DownloadError::DownloadFailed(err_description));
                }
            }
        };

        progress_bar.finish_and_clear();

        // build the DownloadResult dependent on the loop result
        match download_result {
            Ok(_) => {
                // Measuring the time where download is done
                let end = Instant::now();

                // calculate the total download time
                let total_duration = end - start;

                println!(
                    "\nDownload done in   : {}",
                    utils::calc_duration(total_duration.as_secs())
                );

                Ok(final_file_path)
            }
            Err(download_err) => match download_err {
                DownloadError::DownloadInterrupted => {
                    log::debug!("{} was interrupted by user...", app::APP_NAME);
                    println!("{}", app::APP_INTERRUPTED_MSG);
                    // terminate app
                    std::process::exit(1);
                }
                _ => Err(color_eyre::eyre::eyre!(download_err.to_string())),
            },
        }
    }
}
