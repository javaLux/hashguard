use std::{
    cmp::min,
    error::Error,
    fmt,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crate::util;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug)]
struct DownloadError {
    err_description: String,
}

impl DownloadError {
    fn new(err_description: String) -> Self {
        Self { err_description }
    }
}

impl Error for DownloadError {}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DownloadError: {}", self.err_description)
    }
}

#[derive(Debug)]
pub struct DownloadResult {
    download_err: Option<DownloadError>,
    duration: Option<Duration>,
}

/// Execute the file download for the specified URL
pub fn make_download_req(url: &str, final_file_path: &PathBuf) -> color_eyre::Result<()> {
    let http_agent = ureq::builder().build();

    // make a request to the download server
    let download_req = http_agent.get(url).call();

    match download_req {
        Ok(response) => {
            // try to get the Content-Length from the server
            let content_length_header = response.header("Content-Length");

            match content_length_header {
                Some(content_length) => {
                    // try to parse content-length string into a number
                    let content_length = content_length.parse::<usize>();

                    match content_length {
                        Ok(content_length) => {
                            if content_length < 1 {
                                let err_description =
                                    "Download failed - The server sent a content-length of zero"
                                        .to_string();
                                Err(DownloadError::new(err_description).into())
                            } else {
                                // use a channel for thread safe communication
                                let (sender, receiver) = mpsc::channel();

                                let mut downloaded_bytes: usize = 0;

                                // buffer size 4KiB - 4096 bytes
                                let mut buffer = [0; 4096];

                                // create the file to write in
                                let file = File::create(final_file_path)?;

                                let mut writer = BufWriter::new(file);

                                // capture response reader to read bytes from HTTP body
                                let mut body = response.into_reader();

                                // build the progress bar
                                let progress_bar = ProgressBar::new(content_length as u64);
                                progress_bar.set_style(ProgressStyle::with_template("[{msg}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                                    .progress_chars("#>-"));

                                // Start measuring time before read from the http response
                                let start = Instant::now();

                                // open a new thread to handle the download process
                                // if the download process fails, we return the type color-eyr::Result<T> from the closure
                                let download_thread = thread::spawn(move || {
                                    // set progress bar msg
                                    progress_bar.set_message("Download in progress");

                                    let download_result: Result<(), DownloadError> = loop {
                                        // try to read from the response body
                                        let read_result = body.read(&mut buffer);

                                        match read_result {
                                            Ok(bytes_read) => {
                                                // try to write read bytes into the BufWriter
                                                let write_result =
                                                    writer.write_all(&buffer[..bytes_read]);

                                                if let Err(write_err) = write_result {
                                                    let err_description =
                                                format!("Download failed - Unable to write data into file - Details: {:?}", write_err);
                                                    break Err(DownloadError::new(err_description));
                                                }

                                                // capture the successfully downloaded bytes
                                                downloaded_bytes += bytes_read;

                                                // get the right value for the progress bar
                                                let pb_value =
                                                    min(downloaded_bytes, content_length);

                                                progress_bar.set_position(pb_value as u64);

                                                // Break the loop if there are no more bytes to read
                                                if bytes_read == 0 {
                                                    break Ok(());
                                                }
                                            }
                                            Err(body_access_err) => {
                                                let err_description =
                                                format!("Download failed - Could not read from the server response - Details: {:?}", body_access_err);
                                                break Err(DownloadError::new(err_description));
                                            }
                                        }
                                    };

                                    progress_bar.finish_and_clear();

                                    // Measuring the time where download is done
                                    let end = Instant::now();

                                    // calculate the total download time
                                    let total_duration = end - start;

                                    // build the DownloadResult dependent on the loop result
                                    let download_result = match download_result {
                                        Ok(_) => DownloadResult {
                                            download_err: None,
                                            duration: Some(total_duration),
                                        },
                                        Err(download_err) => DownloadResult {
                                            download_err: Some(download_err),
                                            duration: None,
                                        },
                                    };

                                    // send message to the main thread
                                    sender.send(download_result).expect("Couldn't send 'DownloadResult' via channel to the main thread");
                                });

                                // block the main thread until the associated thread is finished
                                download_thread
                                    .join()
                                    .expect("Couldn't join on the 'download' thread");

                                // try to get the message from the channel
                                let download_result = receiver.recv()?;

                                // if the download failed, return the specific error type
                                if let Some(download_err) = download_result.download_err {
                                    return Err(download_err.into());
                                }

                                if let Some(duration) = download_result.duration {
                                    println!(
                                        "Download done in   : {}",
                                        util::calc_duration(duration.as_secs())
                                    );
                                }

                                Ok(())
                            }
                        }
                        Err(parse_err) => {
                            let err_description = format!(
                                "Download failed - The server sent an invalid content-length - Details: {:?}",
                                parse_err
                            );
                            Err(DownloadError::new(err_description).into())
                        }
                    }
                }
                None => {
                    let err_description =
                        "Download failed - The server did not send a content-length header"
                            .to_string();
                    Err(DownloadError::new(err_description).into())
                }
            }
        }
        Err(req_err) => {
            // if the download failed - fetch the concrete error type
            match req_err {
                ureq::Error::Status(code, response_err) => {
                    let err_description = format!(
                        "Download failed - HTTP-Status: {}, Response: {:?}",
                        code, response_err
                    );
                    Err(DownloadError::new(err_description).into())
                }
                ureq::Error::Transport(transport_err) => {
                    let err_description = format!(
                        "Download failed due to HTTP transport error: {:?}",
                        transport_err
                    );
                    Err(DownloadError::new(err_description).into())
                }
            }
        }
    }
}
