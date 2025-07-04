mod cli;

use anyhow::{Context, Result};
use pnger::{embed_payload_from_file_with_options, extract_payload_from_file_with_options};
use std::fs;
use std::io::{self, Write};

use crate::cli::Cli;

macro_rules! log {
    ($level:ident($($arg:tt)+)) => {
        #[cfg(feature = "bin_log")]
        log::$level!($($arg)*);
    };
}

fn embed_payload(args: &Cli, payload_data: &[u8]) -> Result<Vec<u8>> {
    let options = args.get_options()?;
    embed_payload_from_file_with_options(
        args.input
            .to_str()
            .context("Input file path contains invalid UTF-8")?,
        payload_data,
        options,
    )
    .context("Failed to embed payload into PNG")
}

fn extract_payload(args: &Cli) -> Result<Vec<u8>> {
    let options = args.get_options()?;
    extract_payload_from_file_with_options(
        args.input
            .to_str()
            .context("Input file path contains invalid UTF-8")?,
        options,
    )
    .context("Failed to extract payload from PNG")
}

fn write_result(args: &Cli, result: &[u8]) -> Result<()> {
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
    #[cfg(feature = "bin_log")]
    env_logger::init();

    let args = Cli::parse_and_validate()?;
    let result = if args.extract {
        log!(info("Extracting payload from {:?}", args.input));
        extract_payload(&args)?
    } else {
        log!(info(
            "Embedding payload of {:?} into {:?}",
            args.payload,
            args.input
        ));
        let payload_file = &args.payload.clone().expect("payload has to be specified");
        let payload_data = fs::read(payload_file)
            .with_context(|| format!("Failed to read payload file '{payload_file:?}'"))?;
        embed_payload(&args, &payload_data)?
    };
    write_result(&args, &result)?;
    Ok(())
}
