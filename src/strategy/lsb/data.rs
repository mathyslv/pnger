use crate::PayloadSize;
use crate::strategy::lsb::utils::{embed_bit, extract_bit};
use crate::strategy::lsb::{BitIndex, RuntimePattern};
use rand::SeedableRng;
use rand::seq::SliceRandom;

pub(super) struct BodyEmbedder<'a> {
    target_bit_index: BitIndex,
    index: usize,
    indices: Vec<PayloadSize>,
    bytes: &'a mut [u8],
}

impl<'a> BodyEmbedder<'a> {
    pub fn new(
        bytes: &'a mut [u8],
        pattern: &RuntimePattern,
        bit_index: BitIndex,
        payload_len: usize,
    ) -> Self {
        let mut ordered_indices: Vec<u32> = (0..bytes.len()).map(|i| i as u32).collect();
        let indices = match &pattern {
            RuntimePattern::Linear => ordered_indices,
            RuntimePattern::Random { seed, .. } => {
                let mut rng = rand_chacha::ChaCha20Rng::from_seed(*seed);
                let (shuffled, _) = ordered_indices.partial_shuffle(&mut rng, payload_len * 8);
                shuffled.to_vec()
            }
        };

        Self {
            target_bit_index: bit_index,
            index: 0,
            indices,
            bytes,
        }
    }

    pub fn embed_payload(&mut self, payload: &[u8]) {
        let mut indices = self.indices.clone();
        indices.truncate(payload.len() * 8);
        payload.iter().for_each(|byte| self.write_u8(*byte));
    }

    pub fn extract_payload(&mut self, size: usize) -> Vec<u8> {
        let mut indices = self.indices.clone();
        indices.truncate(size * 8);
        let mut payload = Vec::with_capacity(size);
        for _ in 0..size {
            payload.push(self.read_u8());
        }
        payload
    }

    pub fn write_u8(&mut self, byte: u8) {
        let target_bit = self.target_bit_index;

        for bit_pos in 0..8 {
            assert!(
                (self.index < self.indices.len()),
                "LSB index {} is out of bounds (max: {}). Payload too large for available capacity.",
                self.index,
                self.indices.len()
            );

            let image_index = self.indices[self.index] as usize;
            let bit = (byte >> bit_pos) & 1;
            self.bytes[image_index] = embed_bit(target_bit, self.bytes[image_index], bit);
            self.index += 1;
        }
    }

    pub fn read_u8(&mut self) -> u8 {
        let target_bit = self.target_bit_index;
        let mut byte = 0u8;

        for bit_pos in 0..8 {
            assert!(
                (self.index < self.indices.len()),
                "LSB index {} is out of bounds (max: {}). Extraction beyond available data.",
                self.index,
                self.indices.len()
            );

            let image_index = self.indices[self.index] as usize;
            let bit = extract_bit(target_bit, self.bytes[image_index]);
            byte |= (bit & 1) << bit_pos;
            self.index += 1;
        }

        byte
    }
}
