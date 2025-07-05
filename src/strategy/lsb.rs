use std::io::Write;

use crate::error::PngerError;

/// LSB (Least Significant Bit) strategy implementation
pub struct LSBStrategy<'a> {
    index: usize,
    image_data: &'a mut [u8],
    target_bit_index: u8,
}

impl<'a> LSBStrategy<'a> {
    pub fn new(image_data: &'a mut [u8], target_bit_index: u8) -> Self {
        Self {
            index: 0,
            image_data,
            target_bit_index,
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
            self.image_data[self.index] =
                Self::embed_bit(self.target_bit_index, self.image_data[self.index], bit);
            self.index += 1;
        }
    }

    fn write_u32(&mut self, dword: u32) {
        let _ = self
            .write(&dword.to_be_bytes())
            .expect("unable to write u32");
    }

    fn read_u8(&mut self) -> u8 {
        (0..8).fold(0, |byte, bit_index| {
            let bit = Self::extract_bit(self.target_bit_index, self.image_data[self.index]);
            self.index += 1;
            (bit << bit_index) | byte
        })
    }

    fn read_u32(&mut self) -> u32 {
        u32::from_be_bytes([
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
            self.read_u8(),
        ])
    }

    pub fn embed_payload(mut self, payload_data: &[u8]) -> Result<(), PngerError> {
        let max_capacity = self.max_capacity(self.image_data);
        if payload_data.len() > max_capacity {
            return Err(PngerError::PayloadTooLarge);
        }

        println!(">> payload length is {}", payload_data.len());

        // Embed payload length first (4 bytes)
        self.write_u32(payload_data.len() as u32);

        // Embed payload data
        for &byte in payload_data {
            self.write_u8(byte);
        }

        Ok(())
    }

    pub fn extract_payload(mut self) -> Result<Vec<u8>, PngerError> {
        // Embed payload length first (4 bytes)
        let payload_length = self.read_u32() as usize;
        println!(">> payload length is {}", payload_length);

        let max_capacity = self.max_capacity(self.image_data);
        if payload_length > max_capacity {
            return Err(PngerError::PayloadTooLarge);
        }

        let data = (0..payload_length).map(|_| self.read_u8()).collect();
        Ok(data)
    }

    fn max_capacity(&self, image_data: &[u8]) -> usize {
        (image_data.len() - core::mem::size_of::<usize>()) / core::mem::size_of::<u8>()
    }
}

impl<'a> Write for LSBStrategy<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for byte in buf {
            self.write_u8(*byte);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
