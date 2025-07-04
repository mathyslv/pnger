use super::Strategy;
use crate::error::PngerError;

/// LSB (Least Significant Bit) strategy implementation
#[derive(Debug, Clone)]
pub struct LSBStrategy;

impl Strategy for LSBStrategy {
    fn embed(&self, image_data: &mut [u8], payload_data: &[u8]) -> Result<(), PngerError> {
        let max_capacity = self.max_capacity(image_data);
        if payload_data.len() > max_capacity {
            return Err(PngerError::PayloadTooLarge);
        }

        // Embed payload length first (4 bytes)
        let length_bytes = (payload_data.len() as u32).to_be_bytes();
        let mut bit_index = 0;
        
        // Embed length
        for &byte in &length_bytes {
            for bit_pos in 0..8 {
                let bit = (byte >> bit_pos) & 1;
                image_data[bit_index] = (image_data[bit_index] & 0xFE) | bit;
                bit_index += 1;
            }
        }
        
        // Embed payload data
        for &byte in payload_data {
            for bit_pos in 0..8 {
                let bit = (byte >> bit_pos) & 1;
                image_data[bit_index] = (image_data[bit_index] & 0xFE) | bit;
                bit_index += 1;
            }
        }
        
        Ok(())
    }
    
    fn max_capacity(&self, image_data: &[u8]) -> usize {
        // Reserve 32 bits (4 bytes) for length prefix
        (image_data.len() - 32) / 8
    }
}