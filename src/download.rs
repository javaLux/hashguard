use std::{
    cmp::min,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Instant,
};

use crate::util;
use indicatif::{ProgressBar, ProgressStyle};

/// Execute the file download for the specified URL
pub fn download_file(url: &str, final_file_path: &PathBuf) -> Result<(), String> {
    // call the download URL
    let response = ureq::get(url).call();

    if let Ok(response) = response {
        // check if HTTP request has a 'Content-Length'
        let content_length = response.header("Content-Length");

        if let Some(content_length) = content_length {
            // try to convert content-length to usize
            let content_length = content_length.parse::<usize>();

            // try to start the download
            if let Ok(content_length) = content_length {
                if content_length > 0 {
                    // Sender ans Receiver for a Thread-Safe messaging
                    let (sender, receiver) = mpsc::channel();

                    let mut downloaded_bytes: usize = 0;

                    let mut buffer = [0; 4096];

                    // create file dependent on the download url filename
                    let file = File::create(final_file_path).unwrap();

                    let mut writer = BufWriter::new(file);

                    // capture response reader to read bytes from HTTP body
                    let mut response_reader = response.into_reader();

                    // build the progress bar
                    let progress_bar = ProgressBar::new(content_length as u64);
                    progress_bar.set_style(ProgressStyle::with_template("[{msg}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                        .unwrap()
                        .progress_chars("#>-"));

                    // Start measuring time before read from the http response
                    let start = Instant::now();

                    // open a new thread to handle the download process
                    let download_thread = thread::spawn(move || {
                        // set progress bar download msg
                        progress_bar.set_message("Download in progress");
                        loop {
                            let bytes_read = response_reader.read(&mut buffer).unwrap();

                            // write to BufWriter
                            writer.write_all(&buffer[..bytes_read]).unwrap();

                            // capture the successfully downloaded bytes
                            downloaded_bytes += bytes_read;

                            // get the right value for the progress bar
                            let pb_value = min(downloaded_bytes, content_length);

                            progress_bar.set_position(pb_value as u64);

                            // Break the loop if there are no more bytes to read
                            if bytes_read == 0 {
                                break;
                            }
                        }

                        progress_bar.finish_and_clear();

                        // Measuring the time where download is done
                        let end = Instant::now();

                        // calculate the total download time
                        let total_duration = end - start;

                        // send message to the main thread
                        sender.send(total_duration).unwrap();
                    });

                    // block the main thread until the associated thread is finished
                    download_thread.join().unwrap();

                    if let Ok(duration) = receiver.recv() {
                        println!(
                            "Download done in: {}",
                            util::calc_duration(duration.as_secs())
                        )
                    }

                    Ok(())
                } else {
                    Err("Unable to start download - No Content-Length received".to_string())
                }
            } else {
                Err("Unable to start download - Content-Length is not a number".to_string())
            }
        } else {
            Err("Unable to start download - No Content-Length received".to_string())
        }
    } else {
        Err(format!(
            "{} - {}",
            "Unable to call the URL",
            response.err().unwrap()
        ))
    }
}
