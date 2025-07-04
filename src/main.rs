mod cli;

use anyhow::{Context, Result};
use pnger::embed_payload_from_file_with_mode;
use std::fs;
use std::io::{self, Write};

use crate::cli::Cli;

/// Validate command line arguments
fn validate_args(args: &Cli) -> Result<()> {
    if let Err(msg) = args.validate_output() {
        eprintln!("{}", msg);
        std::process::exit(1);
    }
    Ok(())
}

/// Embed payload into PNG using specified mode
fn embed_payload(args: &Cli, payload_data: &[u8]) -> Result<Vec<u8>> {
    embed_payload_from_file_with_mode(
        args.input
            .to_str()
            .context("Input file path contains invalid UTF-8")?,
        payload_data,
        args.mode.into(),
    )
    .context("Failed to embed payload into PNG")
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
    let args = Cli::parse_args();
    validate_args(&args)?;
    let payload_data = fs::read(&args.payload)
        .with_context(|| format!("Failed to read payload file '{}'", args.payload.display()))?;
    let embedded = embed_payload(&args, &payload_data)?;
    write_output(&args, &embedded)?;
    Ok(())
}
