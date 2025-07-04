use clap::{Parser, ValueEnum};
use pnger::Mode;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ModeArg {
    /// Least Significant Bit strategy
    Lsb,
}

impl From<ModeArg> for Mode {
    fn from(arg: ModeArg) -> Self {
        match arg {
            ModeArg::Lsb => Mode::LSB,
        }
    }
}

#[derive(Parser)]
#[command(name = "pnger")]
#[command(version = "0.1.0")]
#[command(about = "A cross-platform tool for embedding & extracting payloads within PNG files")]
#[command(after_help = "Examples:
    # Embed payload.txt into image.png and save to output.png
    pnger -i image.png -p payload.json -o output.png

    # Use explicit LSB mode
    pnger -i image.png -p payload.bin -o output.png --mode lsb

    # Output raw binary data to stdout
    pnger -i image.png -p payload.txt --raw > output.png")]
pub struct Cli {
    /// Input PNG file
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    /// Payload file to embed
    #[arg(short, long, value_name = "FILE")]
    pub payload: PathBuf,

    /// Output file (write result to file)
    #[arg(short, long, value_name = "FILE", conflicts_with = "raw")]
    pub output: Option<PathBuf>,

    /// Output raw binary data to stdout
    #[arg(long, conflicts_with = "output")]
    pub raw: bool,

    /// Embedding mode to use
    #[arg(short, long, value_enum, default_value_t = ModeArg::Lsb)]
    pub mode: ModeArg,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Validate that either output file or raw output is specified
    pub fn validate_output(&self) -> Result<(), String> {
        if self.output.is_none() && !self.raw {
            return Err("Error: Must specify either --output <FILE> or --raw for output method.\nUse --help for more information.".to_string());
        }
        Ok(())
    }
}
