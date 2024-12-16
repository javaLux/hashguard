#![allow(dead_code)]
use std::{
    cmp::min,
    error::Error,
    fmt,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use color_eyre::eyre::Result;

use crate::{
    app, color_templates::WARN_TEMPLATE_NO_BG_COLOR, filename_handling, os_specifics::OS, utils,
};

use indicatif::{ProgressBar, ProgressStyle};

const CONTENT_LENGTH_HEADER: &str = "Content-Length";
const CONTENT_DISPOSITION_HEADER: &str = "Content-Disposition";
const CONTENT_RANGE_HEADER: &str = "Content-Range";
const TRANSFER_ENCODING_HEADER: &str = "Transfer-Encoding";

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

#[derive(Debug)]
enum FileSizeState {
    Known(usize),
    Unknown,
    Chunked,
}

#[derive(Debug)]
struct FileProperties {
    filename: String,
    filepath: PathBuf,
    total_size: usize,
}

/// Executes the file download for the specified URL and returns the path where the file was saved
/// * Make a HTTP-GET request
/// * Check the server response for errors
/// * Verify the response for the required HTTP headers
/// * Starts a progress bar to display the download progress
/// * Write all bytes from the HTTP response body to a file in 4KiB blocks
pub fn execute_download(download_properties: DownloadProperties) -> Result<PathBuf> {
    let http_agent = ureq::builder()
        .timeout_connect(Duration::from_secs(8))
        .build();

    // make a HTTP-Get request
    let response = match http_agent.get(&download_properties.url).call() {
        Ok(response) => response,
        Err(req_err) => {
            // if the download failed - fetch the concrete error type
            match req_err {
                ureq::Error::Status(code, response_err) => {
                    let download_err = DownloadError::DownloadFailed(format!(
                        "HTTP-Status: {}, Response: {:?}",
                        code, response_err
                    ));

                    log::error!("{}", download_err);

                    return Err(color_eyre::eyre::eyre!(download_err));
                }
                ureq::Error::Transport(transport_err) => {
                    let download_err = DownloadError::DownloadFailed(format!(
                        "due to HTTP transport error: {:?}",
                        transport_err
                    ));

                    log::error!("{}", download_err);

                    return Err(color_eyre::eyre::eyre!(download_err));
                }
            }
        }
    };

    log::debug!("{:?}", response);

    let file_size_state = get_file_size_state(&response)?;

    if let FileSizeState::Unknown = file_size_state {
        let err_description = "The response from the server did not contain any information on how to handle the file size of the file to be downloaded. \
Please check the server or try to download the file from another source.";

        let download_err = DownloadError::DownloadFailed(err_description.to_string());

        log::error!("{}", download_err);
        Err(color_eyre::eyre::eyre!(download_err))
    } else {
        // IMPORTANT: use the url from the response object, because in case of an redirect the
        // url can differ from the request url when the http client has follows redirects.
        let url = response.get_url();

        // get the Content-Disposition header
        let content_disposition = response
            .header(CONTENT_DISPOSITION_HEADER)
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
                        .output("Could not determine a filename from server response")
                );
                println!("Please enter a name for the file to be downloaded");
                filename_handling::enter_and_verify_file_name(&download_properties.os_type)?
            }
        };

        // build the final path under which the file is saved
        let filepath = download_properties.output_target.join(filename.clone());

        // capture the server response body and turn it into a Reader
        let body_reader = response.into_reader();

        match file_size_state {
            FileSizeState::Known(total_size) => {
                let file_props = FileProperties {
                    filename,
                    filepath,
                    total_size,
                };

                make_known_size_download(file_props, body_reader)
            }
            FileSizeState::Chunked => {
                let file_props = FileProperties {
                    filename,
                    filepath,
                    total_size: 0,
                };

                make_streamed_download(file_props, body_reader)
            }
            _ => Ok(PathBuf::new()),
        }
    }
}

/// Continue downloading a file of unknown size
fn make_streamed_download(
    file_props: FileProperties,
    mut body_reader: Box<dyn Read + Send + Sync + 'static>,
) -> Result<PathBuf> {
    // buffer size 4KiB - 4096 bytes
    let mut buffer = [0; 4096];

    // create the file to write in
    let file = File::create(&file_props.filepath)?;

    let mut writer = BufWriter::new(file);

    log::debug!("Start streamed download...");
    log::debug!(
        "Output target: {}",
        utils::get_absolute_path(&file_props.filepath)
    );

    // Build a Spinner-Progress-Bar
    let spinner = ProgressBar::new_spinner();

    // Define the spinner style
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&utils::BOUNCING_BAR)
            .template("{spinner:.white} {msg}")
            .unwrap_or(ProgressStyle::default_spinner()),
    );

    // set spinner tick every 100ms
    spinner.enable_steady_tick(Duration::from_millis(100));

    let mut downloaded_bytes: usize = 0;

    // Start measuring time for the download
    let start = Instant::now();

    let download_result = loop {
        // try to read from the response body
        let read_result = body_reader.read(&mut buffer);

        match read_result {
            Ok(bytes_read) => {
                // try to write read bytes into the BufWriter
                let write_result = writer.write_all(&buffer[..bytes_read]);

                if let Err(write_err) = write_result {
                    let download_err = DownloadError::DownloadFailed(format!(
                        "Unable to write data from server response into file: {}",
                        utils::get_absolute_path(&file_props.filepath),
                    ));

                    log::error!("{} - Details: {:?}", download_err, write_err);
                    break Err(download_err);
                }

                // capture the successfully downloaded bytes
                downloaded_bytes += bytes_read;

                spinner.set_message(format!(
                    "Download in progress... {}",
                    utils::convert_bytes_to_human_readable(downloaded_bytes)
                ));

                // Break the loop if there are no more bytes to read
                if bytes_read == 0 {
                    break Ok(());
                }
            }
            Err(body_access_err) => {
                let download_err = DownloadError::DownloadFailed(
                    "Failed to read data from server response".to_string(),
                );

                log::error!("{} - Details: {:?}", download_err, body_access_err);
                break Err(download_err);
            }
        }
    };

    spinner.finish_and_clear();

    // handle the download result
    handle_download_result(start, download_result)?;

    Ok(file_props.filepath)
}

/// Continue downloading a file of known size
fn make_known_size_download(
    file_props: FileProperties,
    mut body_reader: Box<dyn Read + Send + Sync + 'static>,
) -> Result<PathBuf> {
    // buffer size 4KiB - 4096 bytes
    let mut buffer = [0; 4096];

    // create the file to write in
    let file = File::create(&file_props.filepath)?;

    let mut writer = BufWriter::new(file);

    log::debug!("Start download with known file size...");
    log::debug!(
        "File size: {}",
        utils::convert_bytes_to_human_readable(file_props.total_size)
    );
    log::debug!(
        "Output target: {}",
        utils::get_absolute_path(&file_props.filepath)
    );

    // build the progress bar
    let progress_bar = ProgressBar::new(file_props.total_size as u64);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{msg}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap_or(ProgressStyle::default_bar())
        .progress_chars("#>-"),
    );

    progress_bar.set_message("Download in progress");

    let mut downloaded_bytes: usize = 0;

    // Start measuring time for the download
    let start = Instant::now();

    let download_result = loop {
        // try to read from the response body
        let read_result = body_reader.read(&mut buffer);

        match read_result {
            Ok(bytes_read) => {
                // try to write read bytes into the BufWriter
                let write_result = writer.write_all(&buffer[..bytes_read]);

                if let Err(write_err) = write_result {
                    let download_err = DownloadError::DownloadFailed(format!(
                        "Unable to write data from server response into file: {}",
                        utils::get_absolute_path(&file_props.filepath),
                    ));

                    log::error!("{} - Details: {:?}", download_err, write_err);
                    break Err(download_err);
                }

                // capture the successfully downloaded bytes
                downloaded_bytes += bytes_read;

                // get the right value for the progress bar
                let pb_value = min(downloaded_bytes, file_props.total_size);

                progress_bar.set_position(pb_value as u64);

                // Break the loop if there are no more bytes to read
                if bytes_read == 0 {
                    break Ok(());
                }
            }
            Err(body_access_err) => {
                let download_err = DownloadError::DownloadFailed(
                    "Failed to read data from server response".to_string(),
                );

                log::error!("{} - Details: {:?}", download_err, body_access_err);
                break Err(download_err);
            }
        }
    };

    progress_bar.finish_and_clear();

    // handle the download result
    handle_download_result(start, download_result)?;

    Ok(file_props.filepath)
}

fn handle_download_result(start_time: Instant, result: Result<(), DownloadError>) -> Result<()> {
    match result {
        Ok(_) => {
            // Measuring the time where download is done
            let end = Instant::now();

            // calculate the total download time
            let total_duration = end - start_time;

            println!(
                "\nDownload done in   : {}",
                utils::calc_duration(total_duration.as_secs())
            );

            Ok(())
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

fn get_file_size_state(response: &ureq::Response) -> Result<FileSizeState> {
    let file_size_state = {
        let file_size_state = get_content_length(response)?;
        if let FileSizeState::Unknown = file_size_state {
            let file_size_state = get_content_range(response)?;
            if let FileSizeState::Unknown = file_size_state {
                get_transfer_encoding(response)?
            } else {
                file_size_state
            }
        } else {
            file_size_state
        }
    };

    Ok(file_size_state)
}

/// Try to get the `Content-Length` header value form server response
fn get_content_length(response: &ureq::Response) -> Result<FileSizeState> {
    match response.header(CONTENT_LENGTH_HEADER) {
        Some(header_value) => {
            // Try to parse total size of the file as an unsigned integer
            match header_value.parse::<usize>() {
                Ok(total_size) => {
                    if total_size > 0 {
                        Ok(FileSizeState::Known(total_size))
                    } else {
                        let err = DownloadError::DownloadFailed(
                            "The file size could not be determined from the server response"
                                .to_string(),
                        );
                        log::error!(
                            "{} - Details: {}",
                            err,
                            format!(
                                "The server response contains an invalid value for the file size. It can not be zero - {}: {}",
                                CONTENT_LENGTH_HEADER,
                                total_size
                            )
                        );
                        Err(color_eyre::eyre::eyre!(err))
                    }
                }
                Err(parse_err) => {
                    let err = DownloadError::DownloadFailed(
                        "The file size could not be determined from the server response"
                            .to_string(),
                    );
                    log::error!(
                        "{} - Details: {}",
                        err,
                        format!(
                            "{}: {} --> {}",
                            CONTENT_LENGTH_HEADER, header_value, parse_err
                        )
                    );
                    Err(color_eyre::eyre::eyre!(err))
                }
            }
        }
        None => Ok(FileSizeState::Unknown),
    }
}

/// Try to get the `Content-Range` header value form server response
fn get_content_range(response: &ureq::Response) -> Result<FileSizeState> {
    match response.header(CONTENT_RANGE_HEADER) {
        Some(header_value) => {
            // try to extract total size of the file to be downloaded
            match header_value.split('/').last() {
                Some(total_size) => {
                    // The total size of the file in bytes (or '*' if unknown)
                    if total_size.contains("*") {
                        Ok(FileSizeState::Unknown)
                    } else {
                        // try to get the total size as unsigned integer
                        match total_size.parse::<usize>() {
                            Ok(total_size) => {
                                if total_size > 0 {
                                    Ok(FileSizeState::Known(total_size))
                                } else {
                                    let err = DownloadError::DownloadFailed(
                                        "The file size could not be determined from the server response"
                                            .to_string(),
                                    );
                                    log::error!(
                                        "{} - Details: {}",
                                        err,
                                        format!(
                                            "The server response contains an invalid value for the file size. It can not be zero - {}: {}",
                                            CONTENT_RANGE_HEADER,
                                            total_size
                                        )
                                    );
                                    Err(color_eyre::eyre::eyre!(err))
                                }
                            }
                            Err(parse_err) => {
                                let err = DownloadError::DownloadFailed(
                                    "The file size could not be determined from the server response"
                                        .to_string(),
                                );
                                log::error!(
                                    "{} - Details: {}",
                                    err,
                                    format!(
                                        "{}: {} --> {}",
                                        CONTENT_RANGE_HEADER, total_size, parse_err
                                    )
                                );
                                Err(color_eyre::eyre::eyre!(err))
                            }
                        }
                    }
                }
                None => Ok(FileSizeState::Unknown),
            }
        }
        None => Ok(FileSizeState::Unknown),
    }
}

/// Try to get the `Transfer-Encoding` header value form server response
fn get_transfer_encoding(response: &ureq::Response) -> Result<FileSizeState> {
    match response.header(TRANSFER_ENCODING_HEADER) {
        Some(header_value) => {
            if header_value.contains("chunked") {
                Ok(FileSizeState::Chunked)
            } else {
                Ok(FileSizeState::Unknown)
            }
        }
        None => Ok(FileSizeState::Unknown),
    }
}
