use std::{
    cmp::min,
    error::Error,
    fmt,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Instant,
};

use crate::util;
use color_eyre::Result;
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

/// Execute the file download for the specified URL
pub fn download_file(url: &str, final_file_path: &PathBuf) -> Result<()> {
    // call the download URL
    let download_result = ureq::get(url).call();

    match download_result {
        Ok(response) => {
            // check if HTTP request has a 'Content-Length'
            let header_content_length = response.header("Content-Length");

            if let Some(content_length) = header_content_length {
                // try to parse content-length as usize
                let parse_result = content_length.parse::<usize>();

                match parse_result {
                    Ok(content_length) => {
                        // check if a content length is present
                        if content_length > 0 {
                            // use a channel fro thread safe communication
                            let (sender, receiver) = mpsc::channel();

                            let mut downloaded_bytes: usize = 0;

                            // buffer size 8KiB
                            let mut buffer = [0; 8192];

                            // create the file to write in
                            let file = File::create(final_file_path)?;

                            let mut writer = BufWriter::new(file);

                            // build the progress bar
                            let progress_bar = ProgressBar::new(content_length as u64);
                            progress_bar.set_style(ProgressStyle::with_template("[{msg}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                                .progress_chars("#>-"));

                            // Start measuring time before read from the http response
                            let start = Instant::now();

                            // open a new thread to handle the download process
                            // if the download process fails, we return the type color-eyr::Result<T> from the closure
                            let download_thread = thread::spawn(move || -> Result<()> {
                                // capture response as reader to read bytes from HTTP body
                                let mut response_reader = response.into_reader();

                                // set progress bar download msg
                                progress_bar.set_message("Download in progress");
                                loop {
                                    // try to read from the response body
                                    let read_result = response_reader.read(&mut buffer);

                                    if let Ok(bytes_read) = read_result {
                                        // try to write read bytes into the BufWriter
                                        let write_result = writer.write_all(&buffer[..bytes_read]);

                                        if let Err(write_err) = write_result {
                                            let err_description =
                                                format!("Download failed - Unable to write data into file - {:?}", write_err);
                                            return Err(DownloadError::new(err_description).into());
                                        }

                                        // capture the successfully downloaded bytes
                                        downloaded_bytes += bytes_read;

                                        // get the right value for the progress bar
                                        let pb_value = min(downloaded_bytes, content_length);

                                        progress_bar.set_position(pb_value as u64);

                                        // Break the loop if there are no more bytes to read
                                        if bytes_read < 1 {
                                            break;
                                        }
                                    } else {
                                        let err_description =
                                            "Download failed - Unable to read data from server response".to_string();
                                        return Err(DownloadError::new(err_description).into());
                                    }
                                }

                                progress_bar.finish_and_clear();

                                // Measuring the time where download is done
                                let end = Instant::now();

                                // calculate the total download time
                                let total_duration = end - start;

                                // send message to the main thread
                                sender.send(total_duration)?;

                                Ok(())
                            });

                            // block the main thread until the associated thread is finished
                            let join_result = download_thread
                                .join()
                                .expect("Couldn't join on the download thread");

                            join_result?;

                            // try to get the message from the channel
                            let download_duration = receiver.try_recv()?;

                            println!(
                                "Download done in: {}",
                                util::calc_duration(download_duration.as_secs())
                            );

                            Ok(())
                        } else {
                            let err_description =
                                "Download failed - Content-Length can not be zero".to_string();
                            Err(DownloadError::new(err_description).into())
                        }
                    }
                    Err(parse_err) => {
                        let err_description = format!(
                            "Download failed - Content-Length can not interpreted as number: {:?}",
                            parse_err
                        );
                        Err(DownloadError::new(err_description).into())
                    }
                }
            } else {
                let err_description =
                    "Download failed - Header 'Content-Length' missing".to_string();
                Err(DownloadError::new(err_description).into())
            }
        }
        Err(err) => {
            // if the download failed - fetch the concrete error type
            match err {
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
