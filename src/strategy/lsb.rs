use std::io::{Read, Write};

use super::{LSBOptions, PayloadSize};
use crate::error::PngerError;

/// LSB (Least Significant Bit) strategy implementation
pub struct LSBStrategy<'a> {
    index: usize,
    image_data: &'a mut [u8],
    options: LSBOptions,
}

impl<'a> LSBStrategy<'a> {
    pub fn new(image_data: &'a mut [u8], options: LSBOptions) -> Self {
        Self {
            index: 0,
            image_data,
            options,
        }
    }

    fn embed_bit(target_bit_index: u8, carrier: u8, bit: u8) -> u8 {
        let mask = !(1 << target_bit_index);
        (carrier & mask) | ((bit & 1) << target_bit_index)
    }

    fn extract_bit(target_bit_index: u8, carrier: u8) -> u8 {
        let mask = 1 << target_bit_index;
        (carrier & mask) >> target_bit_index
    }

    fn write_u8(&mut self, byte: u8) {
        for bit_pos in 0..8 {
            let bit = byte >> bit_pos;
            self.image_data[self.index] = Self::embed_bit(
                self.options.target_bit_index,
                self.image_data[self.index],
                bit,
            );
            self.index += 1;
        }
    }

    fn read_u8(&mut self) -> u8 {
        (0..8).fold(0, |byte, bit_index| {
            let bit = Self::extract_bit(self.options.target_bit_index, self.image_data[self.index]);
            self.index += 1;
            (bit << bit_index) | byte
        })
    }

    fn read_payload_size(&mut self) -> PayloadSize {
        PayloadSize::from_be_bytes([
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
        ])
    }

    fn write_payload_size(&mut self, size: PayloadSize) {
        let _ = self
            .write(&size.to_be_bytes())
            .expect("unable to write u32");
    }

    pub fn embed_payload(mut self, payload_data: &[u8]) -> Result<(), PngerError> {
        if payload_data.len() as PayloadSize > self.max_capacity() {
            return Err(PngerError::PayloadTooLarge);
        }
        self.write_payload_size(payload_data.len() as PayloadSize);
        self.write_all(payload_data)?;
        Ok(())
    }

    pub fn extract_payload(mut self) -> Result<Vec<u8>, PngerError> {
        let payload_length = self.read_payload_size();
        if payload_length > self.max_capacity() {
            return Err(PngerError::PayloadTooLarge);
        }
        let mut payload = Vec::with_capacity(payload_length as _);
        self.read_to_end(&mut payload)?;
        Ok(payload)
    }

    fn max_capacity(&self) -> PayloadSize {
        ((self.image_data.len() - core::mem::size_of::<PayloadSize>()) / 8) as PayloadSize
    }
}

impl<'a> Write for LSBStrategy<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        buf.iter().for_each(|byte| self.write_u8(*byte));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> Read for LSBStrategy<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        buf.iter_mut().for_each(|byte| *byte = self.read_u8());
        Ok(buf.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_bit() {
        // Test embedding bit 1 at position 0 (LSB)
        assert_eq!(LSBStrategy::embed_bit(0, 0b00000000, 1), 0b00000001);
        assert_eq!(LSBStrategy::embed_bit(0, 0b00000001, 1), 0b00000001);
    
        // Test embedding bit 0 at position 0 (LSB)
        assert_eq!(LSBStrategy::embed_bit(0, 0b00000001, 0), 0b00000000);
        assert_eq!(LSBStrategy::embed_bit(0, 0b00000000, 0), 0b00000000);
    
        // Test all bit positions (0-7) with bit value 1
        assert_eq!(LSBStrategy::embed_bit(0, 0b00000000, 1), 0b00000001);
        assert_eq!(LSBStrategy::embed_bit(1, 0b00000000, 1), 0b00000010);
        assert_eq!(LSBStrategy::embed_bit(2, 0b00000000, 1), 0b00000100);
        assert_eq!(LSBStrategy::embed_bit(3, 0b00000000, 1), 0b00001000);
        assert_eq!(LSBStrategy::embed_bit(4, 0b00000000, 1), 0b00010000);
        assert_eq!(LSBStrategy::embed_bit(5, 0b00000000, 1), 0b00100000);
        assert_eq!(LSBStrategy::embed_bit(6, 0b00000000, 1), 0b01000000);
        assert_eq!(LSBStrategy::embed_bit(7, 0b00000000, 1), 0b10000000);
    
        // Test all bit positions (0-7) with bit value 0 on a byte with all bits set
        assert_eq!(LSBStrategy::embed_bit(0, 0b11111111, 0), 0b11111110);
        assert_eq!(LSBStrategy::embed_bit(1, 0b11111111, 0), 0b11111101);
        assert_eq!(LSBStrategy::embed_bit(2, 0b11111111, 0), 0b11111011);
        assert_eq!(LSBStrategy::embed_bit(3, 0b11111111, 0), 0b11110111);
        assert_eq!(LSBStrategy::embed_bit(4, 0b11111111, 0), 0b11101111);
        assert_eq!(LSBStrategy::embed_bit(5, 0b11111111, 0), 0b11011111);
        assert_eq!(LSBStrategy::embed_bit(6, 0b11111111, 0), 0b10111111);
        assert_eq!(LSBStrategy::embed_bit(7, 0b11111111, 0), 0b01111111);
    
        // Test with mixed carrier bytes
        assert_eq!(LSBStrategy::embed_bit(0, 0b10101010, 1), 0b10101011);
        assert_eq!(LSBStrategy::embed_bit(0, 0b10101011, 0), 0b10101010);
        assert_eq!(LSBStrategy::embed_bit(4, 0b10101010, 1), 0b10111010);
        assert_eq!(LSBStrategy::embed_bit(4, 0b10111010, 0), 0b10101010);

    
        // Test that input bit values > 1 are properly masked
        assert_eq!(LSBStrategy::embed_bit(0, 0b00000000, 0b11111111), 0b00000001);
        assert_eq!(LSBStrategy::embed_bit(0, 0b00000000, 0b11111110), 0b00000000);
    }

    #[test]
    fn test_extract_bit() {
        // Test extracting bit 1 from position 0 (LSB)
        assert_eq!(LSBStrategy::extract_bit(0, 0b00000001), 1);
        assert_eq!(LSBStrategy::extract_bit(0, 0b00000000), 0);
        
        // Test extracting bit 0 from position 0 (LSB)
        assert_eq!(LSBStrategy::extract_bit(0, 0b00000000), 0);
        assert_eq!(LSBStrategy::extract_bit(0, 0b11111110), 0);
        
        // Test all bit positions (0-7) with bit value 1
        assert_eq!(LSBStrategy::extract_bit(0, 0b00000001), 1);
        assert_eq!(LSBStrategy::extract_bit(1, 0b00000010), 1);
        assert_eq!(LSBStrategy::extract_bit(2, 0b00000100), 1);
        assert_eq!(LSBStrategy::extract_bit(3, 0b00001000), 1);
        assert_eq!(LSBStrategy::extract_bit(4, 0b00010000), 1);
        assert_eq!(LSBStrategy::extract_bit(5, 0b00100000), 1);
        assert_eq!(LSBStrategy::extract_bit(6, 0b01000000), 1);
        assert_eq!(LSBStrategy::extract_bit(7, 0b10000000), 1);
        
        // Test all bit positions (0-7) with bit value 0 on a byte with all bits set except target
        assert_eq!(LSBStrategy::extract_bit(0, 0b11111110), 0);
        assert_eq!(LSBStrategy::extract_bit(1, 0b11111101), 0);
        assert_eq!(LSBStrategy::extract_bit(2, 0b11111011), 0);
        assert_eq!(LSBStrategy::extract_bit(3, 0b11110111), 0);
        assert_eq!(LSBStrategy::extract_bit(4, 0b11101111), 0);
        assert_eq!(LSBStrategy::extract_bit(5, 0b11011111), 0);
        assert_eq!(LSBStrategy::extract_bit(6, 0b10111111), 0);
        assert_eq!(LSBStrategy::extract_bit(7, 0b01111111), 0);
        
        // Test with mixed carrier bytes
        assert_eq!(LSBStrategy::extract_bit(0, 0b10101010), 0);
        assert_eq!(LSBStrategy::extract_bit(0, 0b10101011), 1);
        assert_eq!(LSBStrategy::extract_bit(4, 0b10101010), 0);
        assert_eq!(LSBStrategy::extract_bit(4, 0b10111010), 1);
        assert_eq!(LSBStrategy::extract_bit(4, 0b10001010), 0);
        
        // Test extracting from byte with all bits set
        assert_eq!(LSBStrategy::extract_bit(0, 0b11111111), 1);
        assert_eq!(LSBStrategy::extract_bit(3, 0b11111111), 1);
        assert_eq!(LSBStrategy::extract_bit(7, 0b11111111), 1);
        
        // Test round-trip compatibility with embed_bit
        // Embed a bit and then extract it - should get the same bit back
        let carrier = 0b10101010;
        for bit_pos in 0..8 {
            for bit_val in 0..2 {
                let embedded = LSBStrategy::embed_bit(bit_pos, carrier, bit_val);
                let extracted = LSBStrategy::extract_bit(bit_pos, embedded);
                assert_eq!(extracted, bit_val, "Round-trip failed at position {} with bit {}", bit_pos, bit_val);
            }
        }
    }



    #[test]
    fn test_max_capacity() {
        let mut data = [0u8; 128];
        let data_len = data.len();
        let lsb = LSBStrategy::new(&mut data, LSBOptions::default());
        let expected = ((data_len - std::mem::size_of::<PayloadSize>()) / 8) as PayloadSize;
        assert_eq!(lsb.max_capacity(), expected);
    }
}
