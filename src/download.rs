use std::{
    cmp::min,
    error::Error,
    fmt,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
    sync::{atomic::Ordering, mpsc},
    thread,
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

#[derive(Debug)]
pub struct DownloadResult {
    download_err: Option<DownloadError>,
    duration: Option<Duration>,
    file_path: PathBuf,
}

/// Executes the file download for the specified URL and returns the path where the file was saved
pub fn make_download_req(download_properties: DownloadProperties) -> Result<PathBuf> {
    let http_agent = ureq::builder()
        .timeout_connect(std::time::Duration::from_secs(10))
        .build();

    // make the http request to the download server
    let download_req = http_agent.get(&download_properties.url).call();

    match download_req {
        Ok(response) => {
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

            let content_length = content_length.parse::<usize>();
            match content_length {
                Ok(content_length) => {
                    if content_length < 1 {
                        let err_description = format!(
                            "The server sent a {} header of zero",
                            utils::CONTENT_LENGTH_HEADER
                        );
                        Err(color_eyre::eyre::eyre!(DownloadError::DownloadFailed(
                            err_description
                        )))
                    } else {
                        // use a channel for thread safe communication
                        let (sender, receiver) = mpsc::channel();

                        let mut downloaded_bytes: usize = 0;

                        // buffer size 8KiB - 8192 bytes
                        let mut buffer = [0; 8192];

                        // get the owned final file path, because of output the path in write file error msg
                        let file_path = final_file_path.to_owned();

                        // create the file to write in
                        let file = File::create(&file_path)?;

                        let mut writer = BufWriter::new(file);

                        // capture response reader to read bytes from HTTP body
                        let mut body = response.into_reader();

                        // build the progress bar
                        let progress_bar = ProgressBar::new(content_length as u64);
                        progress_bar.set_style(ProgressStyle::with_template("[{msg}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                                .progress_chars("#>-"));

                        log::debug!("Downloading file with the name: {}", filename,);
                        log::debug!(
                            "Output target: {}",
                            utils::get_absolute_path(&download_properties.output_target)
                        );

                        // Start measuring time before read from the http response
                        let start = Instant::now();

                        // open a new thread to handle the download process
                        // if the download process fails, we return the type color-eyr::Result<T> from the closure
                        let download_thread = thread::spawn(move || {
                            // set progress bar msg
                            progress_bar.set_message("Download in progress");

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
                                                format!("Unable to write data from server response into file '{}' - Details: {:?}", utils::get_absolute_path(&file_path), write_err);
                                            break Err(DownloadError::DownloadFailed(
                                                err_description,
                                            ));
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
                                        let err_description =
                                            format!("Could not read from the server response - Details: {:?}", body_access_err);
                                        break Err(DownloadError::DownloadFailed(err_description));
                                    }
                                }
                            };

                            progress_bar.finish_and_clear();

                            // build the DownloadResult dependent on the loop result
                            let download_result = match download_result {
                                Ok(_) => {
                                    // Measuring the time where download is done
                                    let end = Instant::now();

                                    // calculate the total download time
                                    let total_duration = end - start;

                                    DownloadResult {
                                        download_err: None,
                                        duration: Some(total_duration),
                                        file_path,
                                    }
                                }
                                Err(ref download_err) => match download_err {
                                    DownloadError::DownloadFailed(_) => DownloadResult {
                                        download_err: Some(download_err.clone()),
                                        duration: None,
                                        file_path,
                                    },
                                    DownloadError::DownloadInterrupted => {
                                        log::debug!("{} was interrupted by user...", app::APP_NAME);
                                        println!("{}", app::APP_INTERRUPTED_MSG);
                                        // terminate app
                                        std::process::exit(1);
                                    }
                                },
                            };

                            // send message to the main thread
                            sender.send(download_result).expect(
                                "Couldn't send 'DownloadResult' via channel to the main thread",
                            );
                        });

                        // block the main thread until the associated thread is finished
                        download_thread
                            .join()
                            .expect("Couldn't join on the 'download' thread");

                        // try to get the message from the channel
                        let download_result = receiver.recv()?;

                        // if the download failed, return the specific error type
                        if let Some(download_err) = download_result.download_err {
                            return Err(color_eyre::eyre::eyre!(download_err.to_string()));
                        }

                        if let Some(duration) = download_result.duration {
                            println!(
                                "\nDownload done in   : {}",
                                utils::calc_duration(duration.as_secs())
                            );
                        }

                        Ok(download_result.file_path)
                    }
                }
                Err(parse_err) => {
                    let err_description = format!(
                        "The server sent an invalid content-length - Details: {:?}",
                        parse_err
                    );
                    Err(color_eyre::eyre::eyre!(DownloadError::DownloadFailed(
                        err_description
                    )
                    .to_string()))
                }
            }
        }
        Err(req_err) => {
            // if the download failed - fetch the concrete error type
            match req_err {
                ureq::Error::Status(code, response_err) => {
                    let err_description =
                        format!("HTTP-Status: {}, Response: {:?}", code, response_err);
                    Err(color_eyre::eyre::eyre!(DownloadError::DownloadFailed(
                        err_description
                    )
                    .to_string()))
                }
                ureq::Error::Transport(transport_err) => {
                    let err_description =
                        format!("due to HTTP transport error: {:?}", transport_err);
                    Err(color_eyre::eyre::eyre!(DownloadError::DownloadFailed(
                        err_description
                    )
                    .to_string()))
                }
            }
        }
    }
}
