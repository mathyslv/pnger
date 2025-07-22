use super::BitIndex;

pub(super) fn embed_bit(target_bit_index: BitIndex, carrier: u8, bit: u8) -> u8 {
    let bit_pos = u8::from(target_bit_index);
    let mask = !(1 << bit_pos);
    (carrier & mask) | ((bit & 1) << bit_pos)
}

pub(super) fn extract_bit(target_bit_index: BitIndex, carrier: u8) -> u8 {
    let bit_pos = u8::from(target_bit_index);
    let mask = 1 << bit_pos;
    (carrier & mask) >> bit_pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_bit() {
        // Test embedding bit 1 at position 0 (LSB)
        assert_eq!(embed_bit(BitIndex::Bit0, 0b00000000, 1), 0b00000001);
        assert_eq!(embed_bit(BitIndex::Bit0, 0b00000001, 1), 0b00000001);

        // Test embedding bit 0 at position 0 (LSB)
        assert_eq!(embed_bit(BitIndex::Bit0, 0b00000001, 0), 0b00000000);
        assert_eq!(embed_bit(BitIndex::Bit0, 0b00000000, 0), 0b00000000);

        // Test all bit positions (0-7) with bit value 1
        assert_eq!(embed_bit(BitIndex::Bit0, 0b00000000, 1), 0b00000001);
        assert_eq!(embed_bit(BitIndex::Bit1, 0b00000000, 1), 0b00000010);
        assert_eq!(embed_bit(BitIndex::Bit2, 0b00000000, 1), 0b00000100);
        assert_eq!(embed_bit(BitIndex::Bit3, 0b00000000, 1), 0b00001000);
        assert_eq!(embed_bit(BitIndex::Bit4, 0b00000000, 1), 0b00010000);
        assert_eq!(embed_bit(BitIndex::Bit5, 0b00000000, 1), 0b00100000);
        assert_eq!(embed_bit(BitIndex::Bit6, 0b00000000, 1), 0b01000000);
        assert_eq!(embed_bit(BitIndex::Bit7, 0b00000000, 1), 0b10000000);

        // Test all bit positions (0-7) with bit value 0 on a byte with all bits set
        assert_eq!(embed_bit(BitIndex::Bit0, 0b11111111, 0), 0b11111110);
        assert_eq!(embed_bit(BitIndex::Bit1, 0b11111111, 0), 0b11111101);
        assert_eq!(embed_bit(BitIndex::Bit2, 0b11111111, 0), 0b11111011);
        assert_eq!(embed_bit(BitIndex::Bit3, 0b11111111, 0), 0b11110111);
        assert_eq!(embed_bit(BitIndex::Bit4, 0b11111111, 0), 0b11101111);
        assert_eq!(embed_bit(BitIndex::Bit5, 0b11111111, 0), 0b11011111);
        assert_eq!(embed_bit(BitIndex::Bit6, 0b11111111, 0), 0b10111111);
        assert_eq!(embed_bit(BitIndex::Bit7, 0b11111111, 0), 0b01111111);

        // Test with mixed carrier bytes
        assert_eq!(embed_bit(BitIndex::Bit0, 0b10101010, 1), 0b10101011);
        assert_eq!(embed_bit(BitIndex::Bit0, 0b10101011, 0), 0b10101010);
        assert_eq!(embed_bit(BitIndex::Bit4, 0b10101010, 1), 0b10111010);
        assert_eq!(embed_bit(BitIndex::Bit4, 0b10111010, 0), 0b10101010);

        // Test that input bit values > 1 are properly masked
        assert_eq!(
            embed_bit(BitIndex::Bit0, 0b00000000, 0b11111111),
            0b00000001
        );
        assert_eq!(
            embed_bit(BitIndex::Bit0, 0b00000000, 0b11111110),
            0b00000000
        );

        // Test LSB alias
        assert_eq!(embed_bit(BitIndex::LSB, 0b00000000, 1), 0b00000001);
        assert_eq!(embed_bit(BitIndex::LSB, 0b00000001, 0), 0b00000000);
    }

    #[test]
    fn test_extract_bit() {
        // Test extracting bit 1 from position 0 (LSB)
        assert_eq!(extract_bit(BitIndex::Bit0, 0b00000001), 1);
        assert_eq!(extract_bit(BitIndex::Bit0, 0b00000000), 0);

        // Test extracting bit 0 from position 0 (LSB)
        assert_eq!(extract_bit(BitIndex::Bit0, 0b00000000), 0);
        assert_eq!(extract_bit(BitIndex::Bit0, 0b11111110), 0);

        // Test all bit positions (0-7) with bit value 1
        assert_eq!(extract_bit(BitIndex::Bit0, 0b00000001), 1);
        assert_eq!(extract_bit(BitIndex::Bit1, 0b00000010), 1);
        assert_eq!(extract_bit(BitIndex::Bit2, 0b00000100), 1);
        assert_eq!(extract_bit(BitIndex::Bit3, 0b00001000), 1);
        assert_eq!(extract_bit(BitIndex::Bit4, 0b00010000), 1);
        assert_eq!(extract_bit(BitIndex::Bit5, 0b00100000), 1);
        assert_eq!(extract_bit(BitIndex::Bit6, 0b01000000), 1);
        assert_eq!(extract_bit(BitIndex::Bit7, 0b10000000), 1);

        // Test all bit positions (0-7) with bit value 0 on a byte with all bits set except target
        assert_eq!(extract_bit(BitIndex::Bit0, 0b11111110), 0);
        assert_eq!(extract_bit(BitIndex::Bit1, 0b11111101), 0);
        assert_eq!(extract_bit(BitIndex::Bit2, 0b11111011), 0);
        assert_eq!(extract_bit(BitIndex::Bit3, 0b11110111), 0);
        assert_eq!(extract_bit(BitIndex::Bit4, 0b11101111), 0);
        assert_eq!(extract_bit(BitIndex::Bit5, 0b11011111), 0);
        assert_eq!(extract_bit(BitIndex::Bit6, 0b10111111), 0);
        assert_eq!(extract_bit(BitIndex::Bit7, 0b01111111), 0);

        // Test with mixed carrier bytes
        assert_eq!(extract_bit(BitIndex::Bit0, 0b10101010), 0);
        assert_eq!(extract_bit(BitIndex::Bit0, 0b10101011), 1);
        assert_eq!(extract_bit(BitIndex::Bit4, 0b10101010), 0);
        assert_eq!(extract_bit(BitIndex::Bit4, 0b10111010), 1);
        assert_eq!(extract_bit(BitIndex::Bit4, 0b10001010), 0);

        // Test extracting from byte with all bits set
        assert_eq!(extract_bit(BitIndex::Bit0, 0b11111111), 1);
        assert_eq!(extract_bit(BitIndex::Bit3, 0b11111111), 1);
        assert_eq!(extract_bit(BitIndex::Bit7, 0b11111111), 1);

        // Test LSB alias
        assert_eq!(extract_bit(BitIndex::LSB, 0b00000001), 1);
        assert_eq!(extract_bit(BitIndex::LSB, 0b11111110), 0);
    }

    #[test]
    fn test_embed_extract_bit_round_trip() {
        // Embed a bit and then extract it - should get the same bit back
        let carrier = 0b10101010;
        for &bit_index in BitIndex::all() {
            for bit_val in 0..2 {
                let embedded = embed_bit(bit_index, carrier, bit_val);
                let extracted = extract_bit(bit_index, embedded);
                assert_eq!(
                    extracted, bit_val,
                    "Round-trip failed at position {bit_index:?} with bit {bit_val}"
                );
            }
        }

        // Test specifically with LSB alias
        let embedded = embed_bit(BitIndex::LSB, carrier, 1);
        let extracted = extract_bit(BitIndex::LSB, embedded);
        assert_eq!(extracted, 1);
    }
}
