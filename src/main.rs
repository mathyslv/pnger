mod cli;

use anyhow::{Context, Result};
use pnger::{embed_payload_from_file_with_mode, extract_payload_from_file_with_mode};
use std::fs;
use std::io::{self, Write};

use crate::cli::Cli;

/// Embed payload into PNG using specified mode
fn embed_payload(args: &Cli, payload_data: &[u8]) -> Result<Vec<u8>> {
    embed_payload_from_file_with_mode(
        args.input
            .to_str()
            .context("Input file path contains invalid UTF-8")?,
        payload_data,
        args.get_mode(),
    )
    .context("Failed to extract payload from PNG")
}

/// Extract payload from PNG using specified mode
fn extract_payload(args: &Cli) -> Result<Vec<u8>> {
    extract_payload_from_file_with_mode(
        args.input
            .to_str()
            .context("Input file path contains invalid UTF-8")?,
        args.get_mode(),
    )
    .context("Failed to extract payload into PNG")
    .map(|(payload, _)| payload)
}

/// Write result to specified output
fn write_output(args: &Cli, result: &[u8]) -> Result<()> {
    if let Some(output_path) = args.output.as_ref() {
        fs::write(output_path, result)
            .with_context(|| format!("Failed to write output file '{}'", output_path.display()))?;
        println!(
            "Successfully embedded payload. Output written to: {}",
            output_path.display()
        );
    } else if args.raw {
        io::stdout()
            .write_all(result)
            .context("Failed to write binary data to stdout")?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse_and_validate()?;
    let result = if args.extract {
        extract_payload(&args)?
    } else {
        let payload_file = &args.payload.clone().expect("payload has to be specified");
        let payload_data = fs::read(payload_file)
            .with_context(|| format!("Failed to read payload file '{:?}'", payload_file))?;
        embed_payload(&args, &payload_data)?
    };
    write_output(&args, &result)?;
    Ok(())
}
