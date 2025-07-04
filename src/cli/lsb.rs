use clap::ValueEnum;
use pnger::strategy::lsb::LSBConfig;

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
            if index > 7 {
                return Err(format!("Bit index must be 0-7, got {index}"));
            }
            config = config.with_bit_index(index);
        }

        // Apply password or seed for random patterns
        if let LSBPatternArg::Random = self {
            if let Some(password) = password {
                config = config.with_password(password);
            } else if let Some(seed) = seed {
                if seed.len() != 32 {
                    return Err(format!("Seed must be exactly 32 bytes, got {}", seed.len()));
                }
                let seed_array: [u8; 32] = seed
                    .try_into()
                    .map_err(|_| "Failed to convert seed to array")?;
                config = config.with_seed(seed_array);
            }
            // If neither password nor seed provided, use auto (default)
        } else if password.is_some() || seed.is_some() {
            return Err("Password and seed options are only valid for random patterns".to_string());
        }

        Ok(config)
    }
}
