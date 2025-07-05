pub mod lsb;

use anyhow::bail;
use clap::{Parser, ValueEnum};
use pnger::Mode;
use std::path::PathBuf;

use self::lsb::{LSBCliOptions, LSBPatternArg};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ModeArg {
    /// Least Significant Bit strategy
    Lsb,
}

#[derive(Parser)]
#[command(name = "pnger")]
#[command(version = "0.1.0")]
#[command(about = "A cross-platform tool for embedding & extracting payloads within PNG files")]
#[command(after_help = "Examples:
    # Embed payload.json into image.png and save to output.png
    pnger -i image.png -p payload.json -o output.png

    # Use explicit LSB mode
    pnger -i image.png -p payload.bin -o output.png --mode lsb

    # Output raw binary data to stdout
    pnger -i image.png -p payload.txt --raw > output.png

    # Extract payload from image.png and save to payload.json
    pnger -x -i output.png -o payload.json

    # Extract payload to stdout
    pnger -x -i output.png --raw")]
pub struct Cli {
    /// Input PNG file
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    /// Payload file to embed
    #[arg(short, long, value_name = "FILE")]
    pub payload: Option<PathBuf>,

    /// Output file (write result to file)
    #[arg(short, long, value_name = "FILE", conflicts_with = "raw")]
    pub output: Option<PathBuf>,

    /// Output raw result data to stdout
    #[arg(long, conflicts_with = "output")]
    pub raw: bool,

    /// Embedding mode to use
    #[arg(short, long, value_enum, default_value_t = ModeArg::Lsb)]
    pub mode: ModeArg,

    /// Extract payload from input file
    #[arg(short = 'x', long)]
    pub extract: bool,

    /// LSB pattern to use (linear or random) [default: random]
    #[arg(long, value_enum)]
    pub lsb_pattern: Option<LSBPatternArg>,

    /// LSB target bit index (0-7) [default: 0]
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=7))]
    pub lsb_bit_index: Option<u8>,

    /// LSB random seed for reproducible random patterns [default: none]
    #[arg(long)]
    pub lsb_seed: Option<u64>,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_and_validate() -> anyhow::Result<Self> {
        let cli = Self::parse();
        cli.validate().map(|_| cli)
    }

    /// Convert CLI arguments to Mode with options
    pub fn get_mode(&self) -> Mode {
        match self.mode {
            ModeArg::Lsb => {
                let lsb_options = LSBCliOptions {
                    pattern: self.lsb_pattern,
                    bit_index: self.lsb_bit_index,
                    seed: self.lsb_seed,
                };
                Mode::LSB(lsb_options.to_options())
            }
        }
    }

    fn validate(&self) -> anyhow::Result<()> {
        // either --output or --raw must be specified
        if self.output.is_none() && !self.raw {
            bail!("Error: must specify either --output <FILE> or --raw for output method.\nUse --help for more information.");
        }

        if !self.extract && self.payload.is_none() {
            bail!("Error: a payload file has to be specified with --payload")
        }

        Ok(())
    }
}
