pub mod lsb;

use anyhow::bail;
use clap::{Parser, ValueEnum};
use pnger::{
    EmbeddingOptions, Obfuscation,
    strategy::{Strategy, lsb::SEED_SIZE},
};
use std::path::PathBuf;

use lsb::LSBPatternArg;

const PNGER_DEFAULT_XOR_KEY: &str = "PNGER_DEFAULT_XOR_KEY";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StrategyArg {
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

    # LSB with password-based pattern (secure, nothing embedded)
    pnger -i image.png -p secret.txt -o output.png --lsb-password \"mypassword123\"

    # LSB with manual hex seed (32 bytes = 64 hex chars)
    pnger -i image.png -p data.bin -o output.png --lsb-seed \"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"

    # LSB linear pattern instead of random
    pnger -i image.png -p data.txt -o output.png --lsb-pattern linear

    # LSB with custom bit index (target bit 3 instead of 0)
    pnger -i image.png -p secret.bin -o output.png --lsb-bit-index 3

    # XOR obfuscation with default key
    pnger -i image.png -p sensitive.txt -o output.png --xor

    # XOR obfuscation with custom key
    pnger -i image.png -p data.json -o output.png --xor --xor-key \"mykey123\"

    # Combined: LSB password + XOR
    pnger -i image.png -p payload.bin -o output.png --lsb-password \"mypassword\" --xor --xor-key \"encrypt\"

    # Extract payload from image.png and save to payload.json
    pnger -x -i output.png -o payload.json

    # Extract with matching LSB and XOR parameters
    pnger -x -i output.png -o extracted.txt --lsb-password \"mypassword\" --xor --xor-key \"encrypt\"

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

    /// Embedding strategy to use
    #[arg(short, long, value_enum, default_value_t = StrategyArg::Lsb)]
    pub strategy: StrategyArg,

    /// Extract payload from input file
    #[arg(short = 'x', long)]
    pub extract: bool,

    /// Toggle payload obfuscation with XOR algorithm. By default a hard-coded value is used. The --xor-key flag can be used to customize the XOR key
    #[arg(long)]
    pub xor: bool,

    /// Key to use for XOR obfuscation
    #[arg(long)]
    pub xor_key: Option<String>,

    /// LSB pattern to use (linear or random) [default: random]
    #[arg(long, value_enum)]
    pub lsb_pattern: Option<LSBPatternArg>,

    /// LSB target bit index (0-7) [default: 0]
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=7))]
    pub lsb_bit_index: Option<u8>,

    /// Password for reproducible random patterns (nothing embedded in PNG) [default: none]
    #[arg(long)]
    pub lsb_password: Option<String>,

    /// LSB seed for reproducible random patterns (raw 32-byte hex seed) [default: none]
    #[arg(long)]
    pub lsb_seed: Option<String>,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_and_validate() -> anyhow::Result<Self> {
        let cli = Self::parse();
        cli.validate().map(|_| cli)
    }

    /// Convert CLI arguments to Strategy
    fn get_strategy(&self) -> anyhow::Result<Strategy> {
        match self.strategy {
            StrategyArg::Lsb => {
                let pattern = self.lsb_pattern.unwrap_or(LSBPatternArg::Random);

                // Parse hex seed if provided
                let seed = if let Some(seed_hex) = &self.lsb_seed {
                    let seed_bytes = hex::decode(seed_hex)
                        .map_err(|e| anyhow::anyhow!("Invalid hex seed: {}", e))?;
                    if seed_bytes.len() != SEED_SIZE {
                        bail!(
                            "Seed must be exactly {} bytes ({} hex characters), got {}",
                            SEED_SIZE,
                            SEED_SIZE * 2,
                            seed_bytes.len()
                        );
                    }
                    Some(seed_bytes)
                } else {
                    None
                };

                let lsb_config = pattern
                    .to_lsb_config(self.lsb_password.clone(), seed, self.lsb_bit_index)
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                Ok(Strategy::LSB(lsb_config))
            }
        }
    }

    fn get_obfuscation(&self) -> Option<Obfuscation> {
        if self.xor {
            Some(Obfuscation::Xor {
                key: self
                    .xor_key
                    .clone()
                    .unwrap_or(PNGER_DEFAULT_XOR_KEY.to_owned())
                    .into_bytes(),
            })
        } else {
            None
        }
    }

    pub fn get_options(&self) -> Result<EmbeddingOptions, anyhow::Error> {
        let strategy = self.get_strategy()?;
        let mut options = EmbeddingOptions::new(strategy);
        if let Some(obfuscation) = self.get_obfuscation() {
            options.set_obfuscation(Some(obfuscation));
        }
        Ok(options)
    }

    fn validate(&self) -> anyhow::Result<()> {
        // either --output or --raw must be specified
        if self.output.is_none() && !self.raw {
            bail!(
                "Error: must specify either --output <FILE> or --raw for output method.\nUse --help for more information."
            );
        }

        if !self.extract && self.payload.is_none() {
            bail!("Error: a payload file has to be specified with --payload")
        }

        Ok(())
    }
}
