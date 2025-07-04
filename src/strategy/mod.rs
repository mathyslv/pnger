use crate::error::PngerError;

/// Available modes for payload insertion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Least Significant Bit strategy
    #[default]
    LSB,
}

/// Trait for different steganography strategies
pub trait Strategy {
    /// Embed payload data into image data
    fn embed(&self, image_data: &mut [u8], payload_data: &[u8]) -> Result<(), PngerError>;
    
    /// Calculate maximum payload capacity for given image data
    fn max_capacity(&self, image_data: &[u8]) -> usize;
}

/// Get strategy implementation for a given mode
pub fn get_strategy(mode: Mode) -> Box<dyn Strategy> {
    match mode {
        Mode::LSB => Box::new(lsb::LSBStrategy),
    }
}

pub mod lsb;