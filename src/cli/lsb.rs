use clap::ValueEnum;
use pnger::strategy::lsb::{BitIndex, LSBConfig, SEED_SIZE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LSBPatternArg {
    /// Linear pattern (sequential)
    Linear,
    /// Random pattern (pseudo-random)
    Random,
}

impl LSBPatternArg {
    /// Convert CLI argument to LSBConfig using the new builder pattern
    pub fn to_lsb_config(
        self,
        password: Option<String>,
        seed: Option<Vec<u8>>,
        bit_index: Option<u8>,
    ) -> Result<LSBConfig, String> {
        let mut config = match self {
            LSBPatternArg::Linear => LSBConfig::linear(),
            LSBPatternArg::Random => LSBConfig::random(),
        };

        // Apply bit index if provided
        if let Some(index) = bit_index {
            let bit_index = BitIndex::try_from(index)
                .map_err(|_| format!("Bit index must be 0-7, got {index}"))?;
            config = config.with_bit_index(bit_index);
        }

        // Apply password or seed for random patterns
        if let LSBPatternArg::Random = self {
            if let Some(password) = password {
                config = config.with_password(password);
            } else if let Some(seed) = seed {
                if seed.len() != SEED_SIZE {
                    return Err(format!(
                        "Seed must be exactly {} bytes, got {}",
                        SEED_SIZE,
                        seed.len()
                    ));
                }
                let seed_array: [u8; SEED_SIZE] = seed.try_into().map_err(|v: Vec<u8>| {
                    format!(
                        "Failed to convert seed to array: expected {} bytes, got {} bytes",
                        SEED_SIZE,
                        v.len()
                    )
                })?;
                config = config.with_seed(seed_array);
            }
            // If neither password nor seed provided, use auto (default)
        } else if password.is_some() || seed.is_some() {
            return Err("Password and seed options are only valid for random patterns".to_string());
        }

        Ok(config)
    }
}
