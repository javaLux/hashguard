use anyhow::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::{command_handling::CommandResult, utils};

pub const BOUNCING_BAR: [&str; 16] = [
    "[    ]", "[=   ]", "[==  ]", "[=== ]", "[====]", "[ ===]", "[  ==]", "[   =]", "[    ]",
    "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
];

pub fn get_stdout(no_color: bool) -> StandardStream {
    if no_color {
        StandardStream::stdout(ColorChoice::Never)
    } else {
        StandardStream::stdout(ColorChoice::Always)
    }
}

fn write_input_source(mut stdout: &mut StandardStream, cmd_result: &CommandResult) -> Result<()> {
    let source = match &cmd_result.file_location {
        Some(file_location) => utils::absolute_path_as_string(file_location),
        None => match &cmd_result.buffer {
            Some(buffer) => format!("Buffer of size {} byte(s)", buffer.len()),
            None => "Buffer of unknown size".to_string(),
        },
    };

    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
    write!(&mut stdout, "\nInput source   : ")?;
    stdout.reset()?;
    writeln!(&mut stdout, "{}", source)?;
    Ok(())
}

fn write_calculated_hash(stdout: &mut StandardStream, hash: &str) -> Result<()> {
    let calculated_hash_sum = format!("Calculated hash: {hash}");

    log::info!("{calculated_hash_sum}");
    writeln!(stdout, "{calculated_hash_sum}")?;
    Ok(())
}

/// Prints the given hash and highlights all differing bytes compared to the calculated hash.
pub fn write_given_hash(
    mut stdout: &mut StandardStream,
    given_hash: &str,
    calculated_hash: &str,
) -> Result<()> {
    // (Hex -> u8)
    let actual_bytes = hex::decode(given_hash)?;
    let expected_bytes = hex::decode(calculated_hash)?;

    // Compare-Bool-Array
    let matches: Vec<bool> = actual_bytes
        .iter()
        .zip(expected_bytes.iter())
        .map(|(a, e)| a == e)
        .collect();

    write!(stdout, "Given hash     : ")?;

    // Only output the differing bytes
    for (byte, is_match) in actual_bytes.iter().zip(matches.iter()) {
        if *is_match {
            write!(&mut stdout, "{:02x}", byte)?;
        } else {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
            write!(&mut stdout, "{:02x}", byte)?;
            stdout.reset()?;
        }
    }

    if actual_bytes.len() > expected_bytes.len() {
        for byte in &actual_bytes[expected_bytes.len()..] {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
            write!(&mut stdout, "{:02x}", byte)?;
            stdout.reset()?;
        }
    }

    writeln!(&mut stdout)?;
    Ok(())
}

fn write_match_status(stdout: &mut StandardStream, is_equal: bool) -> Result<()> {
    let (msg, color) = if is_equal {
        ("Hash sums match", Color::Green)
    } else {
        ("Hash sums DO NOT match", Color::Red)
    };

    stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;
    write!(stdout, "\n{}", msg)?;
    reset_color(stdout)?;

    Ok(())
}

fn write_algorithm(stdout: &mut StandardStream, text: &str, algorithm: &str) -> Result<()> {
    write!(stdout, "{text}")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
    writeln!(stdout, "{}", algorithm)?;
    reset_color(stdout)?;
    writeln!(stdout)?;
    Ok(())
}

pub fn reset_color(stdout: &mut StandardStream) -> Result<()> {
    stdout.reset()?;
    stdout.set_color(&ColorSpec::new())?;
    Ok(())
}

/// Printing the command result
pub fn print_result(cmd_result: &CommandResult, no_color: bool) -> Result<()> {
    let mut output_stream = get_stdout(no_color);

    write_input_source(&mut output_stream, cmd_result)?;
    write_calculated_hash(&mut output_stream, &cmd_result.calculated_hash_sum)?;

    if let Some(hash_to_compare) = &cmd_result.hash_compare_result {
        write_given_hash(
            &mut output_stream,
            &hash_to_compare.given_hash,
            &cmd_result.calculated_hash_sum,
        )?;

        write_match_status(&mut output_stream, hash_to_compare.is_equal)?;
        write_algorithm(
            &mut output_stream,
            " - Used algorithm: ",
            &cmd_result.used_algorithm.to_string(),
        )?;
    } else {
        write_algorithm(
            &mut output_stream,
            "\n- Used algorithm: ",
            &cmd_result.used_algorithm.to_string(),
        )?;
    }

    Ok(())
}
