use clap::ValueEnum;
use pnger::strategy::{LSBOptions, LSBPattern};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LSBPatternArg {
    /// Linear pattern (sequential)
    Linear,
    /// Random pattern (pseudo-random)
    Random,
}

impl From<LSBPatternArg> for LSBPattern {
    fn from(arg: LSBPatternArg) -> Self {
        match arg {
            LSBPatternArg::Linear => LSBPattern::Linear,
            LSBPatternArg::Random => LSBPattern::Random,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LSBCliOptions {
    pub pattern: Option<LSBPatternArg>,
    pub bit_index: Option<u8>,
    pub seed: Option<u64>,
}

impl LSBCliOptions {
    pub fn to_options(&self) -> LSBOptions {
        let defaults = LSBOptions::default();

        LSBOptions {
            pattern: self.pattern.map(|p| p.into()).unwrap_or(defaults.pattern),
            target_bit_index: self.bit_index.unwrap_or(defaults.target_bit_index),
            seed: self.seed.or(defaults.seed),
        }
    }
}
