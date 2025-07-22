//! Bit index enumeration for LSB steganography operations.
//!
//! This module provides a strongly-typed enum for specifying which bit position
//! to target during LSB steganography operations. Using an enum prevents invalid
//! bit indices and makes the API more self-documenting.

/// Enumeration of valid bit indices for LSB steganography.
///
/// Each variant represents a specific bit position in a byte, from the least
/// significant bit (Bit0) to the most significant bit (Bit7). Lower indices
/// provide better visual quality but may be more detectable, while higher
/// indices are more detectable but provide less visual impact.
///
/// # Bit Index Recommendations
///
/// | Variant | Position | Visual Impact | Detectability | Recommended Use |
/// |---------|----------|---------------|---------------|------------------|
/// | Bit0    | 0        | Minimal       | Low           | General purpose  |
/// | Bit1    | 1        | Minimal       | Medium        | Better security  |
/// | Bit2    | 2        | Low           | Medium        | High security    |
/// | Bit3    | 3        | Noticeable    | High          | Not recommended  |
/// | Bit4+   | 4-7      | Very noticeable| Very high    | Not recommended  |
///
/// # Examples
///
/// ```rust
/// use pnger::strategy::lsb::{BitIndex, LSBConfig};
///
/// // Use the least significant bit (most common)
/// let config = LSBConfig::linear().with_bit_index(BitIndex::Bit0);
///
/// // Use the second least significant bit for better security
/// let config = LSBConfig::linear().with_bit_index(BitIndex::Bit1);
///
/// // Use the LSB alias for convenience
/// let config = LSBConfig::linear().with_bit_index(BitIndex::LSB);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitIndex {
    /// Bit position 0 (least significant bit).
    ///
    /// This is the most commonly used bit position as it provides:
    /// - Minimal visual impact on the image
    /// - Good balance between capacity and detectability
    /// - Standard choice for most steganography applications
    Bit0,

    /// Bit position 1.
    ///
    /// Slightly more secure than Bit0 with minimal additional visual impact.
    /// Good choice when you need better security without significant quality loss.
    Bit1,

    /// Bit position 2.
    ///
    /// Higher security with some visual impact. Suitable for applications
    /// where security is more important than perfect visual quality.
    Bit2,

    /// Bit position 3.
    ///
    /// Noticeable visual impact. Use with caution and only when necessary
    /// for specific security requirements.
    Bit3,

    /// Bit position 4.
    ///
    /// Significant visual impact. Generally not recommended for most applications.
    Bit4,

    /// Bit position 5.
    ///
    /// Very significant visual impact. Rarely used in practice.
    Bit5,

    /// Bit position 6.
    ///
    /// Severe visual impact. Use only for specialized applications.
    Bit6,

    /// Bit position 7 (most significant bit).
    ///
    /// Maximum visual impact. Changes the most significant bit which
    /// can dramatically alter pixel values and image appearance.
    Bit7,
}

impl BitIndex {
    /// Convenient alias for the least significant bit (Bit0).
    ///
    /// This is the most commonly used bit position for LSB steganography.
    /// Using this alias makes code more readable and self-documenting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::{BitIndex, LSBConfig};
    ///
    /// // These are equivalent
    /// let config1 = LSBConfig::linear().with_bit_index(BitIndex::LSB);
    /// let config2 = LSBConfig::linear().with_bit_index(BitIndex::Bit0);
    /// assert_eq!(config1.bit_index(), config2.bit_index());
    /// ```
    pub const LSB: BitIndex = BitIndex::Bit0;

    /// Returns all valid bit indices as a slice.
    ///
    /// Useful for iteration or validation purposes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::BitIndex;
    ///
    /// for bit_index in BitIndex::all() {
    ///     println!("Bit position: {}", u8::from(*bit_index));
    /// }
    /// ```
    pub const fn all() -> &'static [BitIndex] {
        &[
            BitIndex::Bit0,
            BitIndex::Bit1,
            BitIndex::Bit2,
            BitIndex::Bit3,
            BitIndex::Bit4,
            BitIndex::Bit5,
            BitIndex::Bit6,
            BitIndex::Bit7,
        ]
    }

    /// Returns the numeric bit position (0-7).
    ///
    /// This method provides a convenient way to get the underlying
    /// bit position without using the `From` trait.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::BitIndex;
    ///
    /// assert_eq!(BitIndex::Bit0.position(), 0);
    /// assert_eq!(BitIndex::Bit3.position(), 3);
    /// assert_eq!(BitIndex::Bit7.position(), 7);
    /// assert_eq!(BitIndex::LSB.position(), 0);
    /// ```
    pub const fn position(self) -> u8 {
        match self {
            BitIndex::Bit0 => 0,
            BitIndex::Bit1 => 1,
            BitIndex::Bit2 => 2,
            BitIndex::Bit3 => 3,
            BitIndex::Bit4 => 4,
            BitIndex::Bit5 => 5,
            BitIndex::Bit6 => 6,
            BitIndex::Bit7 => 7,
        }
    }
}

/// Convert `BitIndex` to u8 for use with low-level bit manipulation functions.
///
/// This conversion is infallible and returns the bit position (0-7).
///
/// # Examples
///
/// ```rust
/// use pnger::strategy::lsb::BitIndex;
///
/// assert_eq!(u8::from(BitIndex::Bit0), 0);
/// assert_eq!(u8::from(BitIndex::Bit3), 3);
/// assert_eq!(u8::from(BitIndex::LSB), 0);
/// ```
impl From<BitIndex> for u8 {
    fn from(bit_index: BitIndex) -> Self {
        bit_index.position()
    }
}

/// Try to convert u8 to `BitIndex`, returning None for invalid values.
///
/// This conversion validates that the input is in the valid range (0-7)
/// and returns the corresponding `BitIndex` variant.
///
/// # Examples
///
/// ```rust
/// use pnger::strategy::lsb::BitIndex;
///
/// assert_eq!(BitIndex::try_from(0u8), Ok(BitIndex::Bit0));
/// assert_eq!(BitIndex::try_from(3u8), Ok(BitIndex::Bit3));
/// assert_eq!(BitIndex::try_from(7u8), Ok(BitIndex::Bit7));
/// assert!(BitIndex::try_from(8u8).is_err());
/// ```
impl TryFrom<u8> for BitIndex {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BitIndex::Bit0),
            1 => Ok(BitIndex::Bit1),
            2 => Ok(BitIndex::Bit2),
            3 => Ok(BitIndex::Bit3),
            4 => Ok(BitIndex::Bit4),
            5 => Ok(BitIndex::Bit5),
            6 => Ok(BitIndex::Bit6),
            7 => Ok(BitIndex::Bit7),
            _ => Err("Bit index must be in range 0-7"),
        }
    }
}

/// Display implementation for `BitIndex`.
///
/// Shows the bit position in a human-readable format.
///
/// # Examples
///
/// ```rust
/// use pnger::strategy::lsb::BitIndex;
///
/// assert_eq!(format!("{}", BitIndex::Bit0), "Bit0");
/// assert_eq!(format!("{}", BitIndex::LSB), "Bit0");
/// ```
impl std::fmt::Display for BitIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_index_position() {
        assert_eq!(BitIndex::Bit0.position(), 0);
        assert_eq!(BitIndex::Bit1.position(), 1);
        assert_eq!(BitIndex::Bit2.position(), 2);
        assert_eq!(BitIndex::Bit3.position(), 3);
        assert_eq!(BitIndex::Bit4.position(), 4);
        assert_eq!(BitIndex::Bit5.position(), 5);
        assert_eq!(BitIndex::Bit6.position(), 6);
        assert_eq!(BitIndex::Bit7.position(), 7);
    }

    #[test]
    fn test_lsb_alias() {
        assert_eq!(BitIndex::LSB, BitIndex::Bit0);
        assert_eq!(BitIndex::LSB.position(), 0);
        assert_eq!(u8::from(BitIndex::LSB), 0);
    }

    #[test]
    fn test_from_bit_index_to_u8() {
        assert_eq!(u8::from(BitIndex::Bit0), 0);
        assert_eq!(u8::from(BitIndex::Bit1), 1);
        assert_eq!(u8::from(BitIndex::Bit2), 2);
        assert_eq!(u8::from(BitIndex::Bit3), 3);
        assert_eq!(u8::from(BitIndex::Bit4), 4);
        assert_eq!(u8::from(BitIndex::Bit5), 5);
        assert_eq!(u8::from(BitIndex::Bit6), 6);
        assert_eq!(u8::from(BitIndex::Bit7), 7);
        assert_eq!(u8::from(BitIndex::LSB), 0);
    }

    #[test]
    fn test_try_from_u8_to_bit_index() {
        // Valid conversions
        assert_eq!(BitIndex::try_from(0u8), Ok(BitIndex::Bit0));
        assert_eq!(BitIndex::try_from(1u8), Ok(BitIndex::Bit1));
        assert_eq!(BitIndex::try_from(2u8), Ok(BitIndex::Bit2));
        assert_eq!(BitIndex::try_from(3u8), Ok(BitIndex::Bit3));
        assert_eq!(BitIndex::try_from(4u8), Ok(BitIndex::Bit4));
        assert_eq!(BitIndex::try_from(5u8), Ok(BitIndex::Bit5));
        assert_eq!(BitIndex::try_from(6u8), Ok(BitIndex::Bit6));
        assert_eq!(BitIndex::try_from(7u8), Ok(BitIndex::Bit7));

        // Invalid conversions
        assert!(BitIndex::try_from(8u8).is_err());
        assert!(BitIndex::try_from(255u8).is_err());
    }

    #[test]
    fn test_all_bit_indices() {
        let all = BitIndex::all();
        assert_eq!(all.len(), 8);
        assert_eq!(all[0], BitIndex::Bit0);
        assert_eq!(all[7], BitIndex::Bit7);

        // Verify all positions are covered
        for (i, &bit_index) in all.iter().enumerate() {
            assert_eq!(bit_index.position(), i as u8);
        }
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", BitIndex::Bit0), "Bit0");
        assert_eq!(format!("{}", BitIndex::Bit3), "Bit3");
        assert_eq!(format!("{}", BitIndex::Bit7), "Bit7");
        assert_eq!(format!("{}", BitIndex::LSB), "Bit0");
    }

    #[test]
    fn test_round_trip_conversion() {
        // Test that converting to u8 and back gives the same result
        for &bit_index in BitIndex::all() {
            let as_u8 = u8::from(bit_index);
            let back_to_enum = BitIndex::try_from(as_u8).unwrap();
            assert_eq!(bit_index, back_to_enum);
        }
    }
}
