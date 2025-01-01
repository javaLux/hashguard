use std::{
    cmp::min,
    collections::BTreeMap,
    error::Error,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use color_eyre::eyre::Result;

use crate::{
    color_templates::WARN_TEMPLATE_NO_BG_COLOR, filename_handling, os_specifics::OS, utils,
};

use indicatif::{ProgressBar, ProgressStyle};

const BUFFER_SIZE: usize = 4096;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(25);

const CONTENT_LENGTH_HEADER: &str = "Content-Length";
const CONTENT_DISPOSITION_HEADER: &str = "Content-Disposition";
const CONTENT_RANGE_HEADER: &str = "Content-Range";
const TRANSFER_ENCODING_HEADER: &str = "Transfer-Encoding";

/// Error type for download operations
#[derive(Debug, Clone)]
struct DownloadError {
    err_msg: String,
}

impl DownloadError {
    fn new(err_msg: String) -> Self {
        Self { err_msg }
    }
}

impl Error for DownloadError {}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Download canceled | {}", self.err_msg)
    }
}

#[derive(Debug)]
pub struct DownloadProperties {
    pub url: String,
    pub output_target: PathBuf,
    pub default_file_name: Option<String>,
    pub os_type: OS,
}

/// Enum to hold the state of the file size
#[derive(Debug)]
enum FileSizeState {
    Known(usize),
    Unknown,
    Chunked,
}

/// Struct to hold the response headers from the server
struct ResponseHeaders<'a> {
    headers: BTreeMap<String, &'a str>,
}

impl std::fmt::Display for ResponseHeaders<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut headers_str = String::new();

        for (name, value) in &self.headers {
            headers_str.push_str(&format!("=> {}: {}\n", name, value));
        }

        write!(f, "{}", headers_str)
    }
}

/// Executes the file download for the specified URL and returns the path where the file was saved
/// * Make a HTTP-GET request
/// * Check the server response for errors
/// * Verify the response for the required HTTP headers
/// * Starts a progress bar to display the download progress
/// * Write all bytes from the HTTP response body to a file in 4KiB blocks
pub fn execute_download(download_properties: DownloadProperties) -> Result<PathBuf> {
    let spinner = ProgressBar::new_spinner()
        .with_message(format!(
            "Connection establishment... Timeout: {}s",
            CONNECTION_TIMEOUT.as_secs()
        ))
        .with_position(25);

    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&utils::BOUNCING_BAR)
            .template("{spinner:.white} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );

    // Set spinner tick every 100ms
    spinner.enable_steady_tick(Duration::from_millis(100));

    let http_agent = ureq::builder().timeout_connect(CONNECTION_TIMEOUT).build();

    // make the HTTP-Get request to the server
    let response = match http_agent.get(&download_properties.url).call() {
        Ok(response) => {
            spinner.finish_and_clear();
            response
        }
        Err(req_err) => {
            spinner.finish_and_clear();

            // if the download failed - handle the concrete error type
            match req_err {
                ureq::Error::Status(code, response) => {
                    let download_err = DownloadError::new(format!(
                        "Server response with status code: {} - msg: {}",
                        code,
                        response.status_text()
                    ));

                    log::error!("{}", download_err);

                    log::debug!("Response headers:\n{}", get_response_headers(&response));

                    match response.into_string() {
                        Ok(response_body) => {
                            if response_body.trim().is_empty() {
                                log::debug!("Response body is empty");
                            } else {
                                log::debug!("Response body:\n{}", response_body);
                            }
                        }
                        Err(body_read_err) => {
                            log::error!(
                                "Response body could not be read - Details: {:?}",
                                body_read_err
                            );
                        }
                    }

                    return Err(color_eyre::eyre::eyre!(download_err));
                }
                ureq::Error::Transport(transport_err) => {
                    log::error!("Download canceled - Details: {:?}", transport_err);

                    let msg = match transport_err.source() {
                        Some(source) => format!("{} - {:?}", transport_err.kind(), source),
                        None => transport_err.kind().to_string(),
                    };

                    return Err(color_eyre::eyre::eyre!(DownloadError::new(msg)));
                }
            }
        }
    };

    log::debug!("Response headers:\n{}", get_response_headers(&response));

    let file_size_state = determine_file_size_state(&response)?;

    if let FileSizeState::Unknown = file_size_state {
        let err_description = "The server response did not contain any information on how to handle the file size of the file to be downloaded. \
Please check the server or try to download the file from another source.";

        let download_err = DownloadError::new(err_description.to_string());

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
        let file_path = download_properties.output_target.join(filename.clone());

        // capture the server response body and turn it into a Reader
        let body_reader = response.into_reader();

        // start the download process
        make_download_req(file_path, body_reader, file_size_state)
    }
}

fn make_download_req(
    file_path: PathBuf,
    mut body_reader: Box<dyn Read + Send + Sync + 'static>,
    file_size_state: FileSizeState,
) -> Result<PathBuf> {
    // Create the file to write in
    let file = File::create(&file_path)?;
    let mut writer = BufWriter::new(file);

    log::info!(
        "Start download - Total file size: {}",
        match file_size_state {
            FileSizeState::Known(total_size) => utils::convert_bytes_to_human_readable(total_size),
            _ => "unknown".to_string(),
        }
    );

    log::info!("Output target: {}", utils::get_absolute_path(&file_path));

    // Build a Progress-Bar or Spinner
    let progress_bar = match file_size_state {
        FileSizeState::Known(total_size) => {
            let pb = ProgressBar::new(total_size as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "[{msg}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
                )
                .unwrap_or(ProgressStyle::default_bar())
                .progress_chars("#>-"),
            );
            pb.set_message("Download in progress");
            pb
        }
        _ => {
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .tick_strings(&utils::BOUNCING_BAR)
                    .template("{spinner:.white} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner()),
            );
            spinner.enable_steady_tick(Duration::from_millis(100));
            spinner
        }
    };

    let mut buffer = [0; BUFFER_SIZE];
    let mut downloaded_bytes: usize = 0;

    // Start measuring time for the download
    let start = Instant::now();

    let download_result = loop {
        // Try to read from the response body
        match body_reader.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break Ok(downloaded_bytes);
                }

                // Try to write read bytes into the BufWriter
                writer
                    .write_all(&buffer[..bytes_read])
                    .map_err(|write_err| {
                        let download_err = DownloadError::new(format!(
                            "Unable to write data from server response into file: {}",
                            utils::get_absolute_path(&file_path),
                        ));
                        log::error!("{} - Details: {:?}", download_err, write_err);
                        download_err
                    })?;

                // Capture the successfully downloaded bytes
                downloaded_bytes += bytes_read;

                match file_size_state {
                    FileSizeState::Known(total_size) => {
                        let pb_value = min(downloaded_bytes, total_size);
                        progress_bar.set_position(pb_value as u64);
                    }
                    _ => {
                        progress_bar.set_message(format!(
                            "Download in progress... {}",
                            utils::convert_bytes_to_human_readable(downloaded_bytes)
                        ));
                    }
                }
            }
            Err(body_access_err) => {
                let download_err =
                    DownloadError::new("Failed to read data from server response".to_string());
                log::error!("{} - Details: {:?}", download_err, body_access_err);
                break Err(download_err);
            }
        }
    };

    progress_bar.finish_and_clear();

    let written_bytes = download_result?;

    // Generate user information
    handle_download_result(start, written_bytes);

    Ok(file_path)
}

fn handle_download_result(start_time: Instant, written_bytes: usize) {
    log::info!(
        "Download finished - Processed file size: {}",
        utils::convert_bytes_to_human_readable(written_bytes)
    );
    // Measuring the time where download is done
    let end = Instant::now();

    // calculate the total download time
    let total_duration = end - start_time;

    println!(
        "\nDownload done in   : {}",
        utils::calc_duration(total_duration.as_secs())
    );
}

/// Determine the file size state from the server response
fn determine_file_size_state(response: &ureq::Response) -> Result<FileSizeState> {
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
                        let err = DownloadError::new(
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
                    let err = DownloadError::new(
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
                    // if the last part of the header value contains a '*' the total size is unknown
                    // for example: Content-Range: bytes 0-1023/*
                    // so we can use a chunked download
                    if total_size.contains("*") {
                        Ok(FileSizeState::Chunked)
                    } else {
                        // try to get the total size as unsigned integer
                        match total_size.parse::<usize>() {
                            Ok(total_size) => {
                                if total_size > 0 {
                                    Ok(FileSizeState::Known(total_size))
                                } else {
                                    let err = DownloadError::new(
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
                                let err = DownloadError::new(
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

/// Extract the response headers from the server response
fn get_response_headers(response: &ureq::Response) -> ResponseHeaders {
    let response_headers = BTreeMap::from_iter(response.headers_names().iter().map(|name| {
        let value = response.header(name).unwrap_or_default();
        (name.to_string(), value)
    }));

    ResponseHeaders {
        headers: response_headers,
    }
}
