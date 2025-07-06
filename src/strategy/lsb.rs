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
        assert_eq!(
            LSBStrategy::embed_bit(0, 0b00000000, 0b11111111),
            0b00000001
        );
        assert_eq!(
            LSBStrategy::embed_bit(0, 0b00000000, 0b11111110),
            0b00000000
        );
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
    }

    #[test]
    fn test_embed_extract_bit_round_trip() {
        // Embed a bit and then extract it - should get the same bit back
        let carrier = 0b10101010;
        for bit_pos in 0..8 {
            for bit_val in 0..2 {
                let embedded = LSBStrategy::embed_bit(bit_pos, carrier, bit_val);
                let extracted = LSBStrategy::extract_bit(bit_pos, embedded);
                assert_eq!(
                    extracted, bit_val,
                    "Round-trip failed at position {} with bit {}",
                    bit_pos, bit_val
                );
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

    #[test]
    fn test_write_u8() {
        // Test writing byte to image data with LSB (bit index 0)
        let mut image_data = [0b11111111u8; 16]; // Start with all bits set
        let mut lsb = LSBStrategy::new(&mut image_data, LSBOptions::default());

        // Write byte 0b10101010 using LSB strategy
        lsb.write_u8(0b10101010);

        // Check that the LSBs of first 8 bytes match the bits of 0b10101010
        // Bit order: LSB first (bit 0, bit 1, bit 2, ..., bit 7)
        assert_eq!(image_data[0] & 1, 0); // bit 0 of 0b10101010 = 0
        assert_eq!(image_data[1] & 1, 1); // bit 1 of 0b10101010 = 1
        assert_eq!(image_data[2] & 1, 0); // bit 2 of 0b10101010 = 0
        assert_eq!(image_data[3] & 1, 1); // bit 3 of 0b10101010 = 1
        assert_eq!(image_data[4] & 1, 0); // bit 4 of 0b10101010 = 0
        assert_eq!(image_data[5] & 1, 1); // bit 5 of 0b10101010 = 1
        assert_eq!(image_data[6] & 1, 0); // bit 6 of 0b10101010 = 0
        assert_eq!(image_data[7] & 1, 1); // bit 7 of 0b10101010 = 1

        // Check that other bits remain unchanged (should still be 1)
        for (i, _) in image_data.iter().enumerate().take(8) {
            assert_eq!(
                image_data[i] & 0b11111110,
                0b11111110,
                "Non-LSB bits changed at index {}",
                i
            );
        }

        // Check that bytes beyond the written byte are unchanged
        for (i, _) in image_data.iter().enumerate().skip(8).take(8) {
            assert_eq!(
                image_data[i], 0b11111111,
                "Byte {} was modified when it shouldn't be",
                i
            );
        }
    }

    #[test]
    fn test_write_u8_different_bit_positions() {
        // Test writing to different bit positions
        for target_bit in 0..8 {
            let mut image_data = [0b11111111u8; 16];
            let options = LSBOptions {
                target_bit_index: target_bit,
                ..Default::default()
            };
            let mut lsb = LSBStrategy::new(&mut image_data, options);

            // Write byte 0b01010101
            lsb.write_u8(0b01010101);

            // Check that the target bits match the pattern

            let clear_mask = !(1 << target_bit);

            for (i, _) in image_data.iter().enumerate().take(8) {
                let expected_bit = (0b01010101 >> i) & 1;
                let actual_bit = (image_data[i] >> target_bit) & 1;
                assert_eq!(
                    actual_bit, expected_bit,
                    "Bit mismatch at byte {} bit {} for target_bit_index {}",
                    i, target_bit, target_bit
                );

                // Check that other bits remain unchanged
                assert_eq!(
                    image_data[i] & clear_mask,
                    clear_mask,
                    "Non-target bits changed at index {} for target_bit_index {}",
                    i,
                    target_bit
                );
            }
        }
    }

    #[test]
    fn test_write_u8_multiple_bytes() {
        let mut image_data = [0u8; 32];
        let mut lsb = LSBStrategy::new(&mut image_data, LSBOptions::default());

        // Write multiple bytes
        let test_bytes = [0b10101010, 0b01010101, 0b11110000, 0b00001111];

        for &byte in &test_bytes {
            lsb.write_u8(byte);
        }

        // Verify each byte was written correctly
        for (byte_idx, &expected_byte) in test_bytes.iter().enumerate() {
            for bit_idx in 0..8 {
                let image_idx = byte_idx * 8 + bit_idx;
                let expected_bit = (expected_byte >> bit_idx) & 1;
                let actual_bit = image_data[image_idx] & 1;
                assert_eq!(
                    actual_bit, expected_bit,
                    "Mismatch in byte {} bit {} (image index {})",
                    byte_idx, bit_idx, image_idx
                );
            }
        }
    }

    #[test]
    fn test_read_u8() {
        // Prepare image data with known bit pattern
        let mut image_data = [0u8; 16];

        // Set up a pattern: LSBs spell out 0b10101010
        // Bit order: LSB first (bit 0, bit 1, bit 2, ..., bit 7)
        image_data[0] = 0b00000000; // bit 0 = 0
        image_data[1] = 0b00000001; // bit 1 = 1
        image_data[2] = 0b00000000; // bit 2 = 0
        image_data[3] = 0b00000001; // bit 3 = 1
        image_data[4] = 0b00000000; // bit 4 = 0
        image_data[5] = 0b00000001; // bit 5 = 1
        image_data[6] = 0b00000000; // bit 6 = 0
        image_data[7] = 0b00000001; // bit 7 = 1

        let mut lsb = LSBStrategy::new(&mut image_data, LSBOptions::default());
        let result = lsb.read_u8();

        assert_eq!(result, 0b10101010);
    }

    #[test]
    fn test_read_u8_different_bit_positions() {
        // Test reading from different bit positions
        for target_bit in 0..8 {
            let mut image_data = [0u8; 16];

            // Set up pattern 0b01010101 at the target bit position
            for (i, byte) in image_data.iter_mut().enumerate().take(8) {
                let bit_value = (0b01010101 >> i) & 1;
                if bit_value == 1 {
                    *byte |= 1 << target_bit;
                }
            }

            let options = LSBOptions {
                target_bit_index: target_bit,
                ..Default::default()
            };
            let mut lsb = LSBStrategy::new(&mut image_data, options);
            let result = lsb.read_u8();

            assert_eq!(
                result, 0b01010101,
                "Failed to read correct byte from target_bit_index {}",
                target_bit
            );
        }
    }

    #[test]
    fn test_read_u8_with_noise() {
        // Test reading with noise in non-target bits
        let mut image_data = [0b11111111u8; 16]; // All bits set as noise

        // Clear LSBs to create pattern 0b10101010
        for (i, byte) in image_data.iter_mut().enumerate().take(8) {
            let bit_value = (0b10101010 >> i) & 1;
            if bit_value == 0 {
                *byte &= 0b11111110; // Clear LSB
            }
            // LSB already set for bit_value == 1
        }

        let mut lsb = LSBStrategy::new(&mut image_data, LSBOptions::default());
        let result = lsb.read_u8();

        assert_eq!(result, 0b10101010);
    }

    #[test]
    fn test_write_read_u8_round_trip() {
        // Test round-trip: write a byte, then read it back
        let test_bytes = [
            0b00000000, 0b11111111, 0b10101010, 0b01010101, 0b11110000, 0b00001111, 0b10011001,
            0b01100110,
        ];

        for &test_byte in &test_bytes {
            let mut image_data = [0u8; 16];

            // Write the byte
            {
                let mut lsb = LSBStrategy::new(&mut image_data, LSBOptions::default());
                lsb.write_u8(test_byte);
            }

            // Read it back
            {
                let mut lsb = LSBStrategy::new(&mut image_data, LSBOptions::default());
                let result = lsb.read_u8();
                assert_eq!(
                    result, test_byte,
                    "Round-trip failed for byte 0b{:08b}",
                    test_byte
                );
            }
        }
    }

    #[test]
    fn test_write_read_u8_round_trip_all_bit_positions() {
        // Test round-trip for all bit positions
        let test_byte = 0b10110101;

        for target_bit in 0..8 {
            let mut image_data = [0b11111111u8; 16]; // Start with noise
            let options = LSBOptions {
                target_bit_index: target_bit,
                ..Default::default()
            };

            // Write the byte
            {
                let mut lsb = LSBStrategy::new(&mut image_data, options.clone());
                lsb.write_u8(test_byte);
            }

            // Read it back
            {
                let mut lsb = LSBStrategy::new(&mut image_data, options);
                let result = lsb.read_u8();
                assert_eq!(
                    result, test_byte,
                    "Round-trip failed for target_bit_index {} with byte 0b{:08b}",
                    target_bit, test_byte
                );
            }
        }
    }

    #[test]
    fn test_write_read_u8_sequential() {
        // Test writing and reading multiple bytes sequentially
        let test_data = [0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF];
        let mut image_data = [0u8; 64]; // Enough space for test data

        // Write all bytes
        {
            let mut lsb = LSBStrategy::new(&mut image_data, LSBOptions::default());
            for &byte in &test_data {
                lsb.write_u8(byte);
            }
        }

        // Read all bytes back
        {
            let mut lsb = LSBStrategy::new(&mut image_data, LSBOptions::default());
            for (i, &expected) in test_data.iter().enumerate() {
                let result = lsb.read_u8();
                assert_eq!(result, expected, "Mismatch at sequential byte {}", i);
            }
        }
    }
}
