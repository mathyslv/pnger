pub mod lsb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LSBPattern {
    Linear,
    Random,
}

impl Default for LSBPattern {
    fn default() -> Self {
        Self::Random
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LSBOptions {
    pub pattern: LSBPattern,
    pub target_bit_index: u8,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    LSB(LSBOptions),
}

impl Default for Mode {
    fn default() -> Self {
        Self::LSB(LSBOptions::default())
    }
}
