use std::path::PathBuf;

use anyhow::Result;

use crate::{
    cli::{DownloadArgs, LocalArgs},
    download::{self, DownloadProperties},
    hasher::{self, Algorithm},
    local, os_specifics, utils,
};

#[derive(Debug)]
pub struct CommandResult {
    pub file_location: Option<PathBuf>,
    pub buffer: Option<String>,
    pub used_algorithm: Algorithm,
    pub calculated_hash_sum: String,
    pub hash_compare_result: Option<HashCompareResult>,
    pub save: bool,
}

#[derive(Debug)]
pub struct HashCompareResult {
    pub is_hash_equal: bool,
    pub origin_hash_sum: String,
}

// Handle the CLI subcommand 'download'
pub fn handle_download_cmd(args: DownloadArgs, os_type: os_specifics::OS) -> Result<()> {
    // fetch the output target
    let output_target = args.output;

    let output_target = match output_target {
        Some(output_target) => output_target,
        // If no output directory was specified
        None => {
            // try to get the default user download folder dependent on the underlying OS
            os_specifics::download_directory()
        }
    };

    // get the download URL
    let download_url = &args.url;

    // check if the given hash sum was prefixed by a hash algorithm, if so ignore the option [-a, --algorithm]
    let algorithm = if let Some(ref hash_property) = args.hash_property {
        match hash_property.algorithm {
            Some(algorithm) => algorithm,
            None => args.algorithm,
        }
    } else {
        args.algorithm
    };

    // build the required DownloadProperties
    let download_properties = DownloadProperties {
        algorithm,
        url: download_url.to_string(),
        output_target,
        default_file_name: args.rename,
        os_type,
    };

    // start the download
    let download_result = download::execute_download(download_properties)?;

    let cmd_result = if let Some(ref hash_property) = args.hash_property {
        let origin_hash_sum = hash_property.hash.clone();
        let is_hash_equal = hasher::is_hash_equal(&origin_hash_sum, &download_result.hash_sum);

        CommandResult {
            file_location: Some(download_result.file_location),
            buffer: None,
            used_algorithm: algorithm,
            calculated_hash_sum: download_result.hash_sum,
            hash_compare_result: Some(HashCompareResult {
                is_hash_equal,
                origin_hash_sum,
            }),
            save: args.save,
        }
    } else {
        CommandResult {
            file_location: Some(download_result.file_location),
            buffer: None,
            used_algorithm: algorithm,
            calculated_hash_sum: download_result.hash_sum,
            hash_compare_result: None,
            save: args.save,
        }
    };
    utils::processing_cmd_result(&cmd_result)?;

    Ok(())
}

// Handle the CLI subcommand 'local'
pub fn handle_local_cmd(args: LocalArgs) -> Result<()> {
    let algorithm = if let Some(ref hash_property) = args.hash_sum {
        match hash_property.algorithm {
            Some(algorithm) => algorithm,
            None => args.algorithm,
        }
    } else {
        args.algorithm
    };

    let (calculated_hash_sum, file_location, buffer) = if let Some(path) = args.path {
        // calculate the file hash
        let calculated_hash_sum =
            local::get_hash_for_object(path.clone(), algorithm, args.include_names)?;
        (calculated_hash_sum, Some(path), None)
    } else if let Some(some_text) = args.buffer {
        let buffer = some_text.as_bytes().to_vec();
        let calculated_hash_sum = local::get_buffer_hash(&buffer, algorithm);
        (calculated_hash_sum, None, Some(some_text))
    } else {
        return Err(anyhow::anyhow!(
            "Either a path or a buffer must be provided."
        ));
    };

    let cmd_result = if let Some(ref hash_property) = args.hash_sum {
        let origin_hash_sum = hash_property.hash.clone();
        let is_hash_equal = hasher::is_hash_equal(&origin_hash_sum, &calculated_hash_sum);

        CommandResult {
            file_location,
            buffer,
            used_algorithm: algorithm,
            calculated_hash_sum: calculated_hash_sum.to_string(),
            hash_compare_result: Some(HashCompareResult {
                is_hash_equal,
                origin_hash_sum,
            }),
            save: args.save,
        }
    } else {
        CommandResult {
            file_location,
            buffer,
            used_algorithm: algorithm,
            calculated_hash_sum: calculated_hash_sum.to_string(),
            hash_compare_result: None,
            save: args.save,
        }
    };
    utils::processing_cmd_result(&cmd_result)?;

    Ok(())
}
