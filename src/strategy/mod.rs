//! Steganography strategy configuration for payload embedding.
//!
//! This module defines the strategy types used to configure how payloads are embedded
//! into PNG images. Currently supports LSB (Least Significant Bit) steganography
//! with plans for additional algorithms in the future.

use crate::strategy::lsb::LSBConfig;

pub mod lsb;

/// Wire format payload size type for cross-platform compatibility.
///
/// Uses 32-bit unsigned integer to ensure consistent behavior across
/// different architectures and platforms.
#[doc(hidden)]
pub type PayloadSize = u32;

/// Steganography embedding strategy selection.
///
/// This enum determines which steganography algorithm will be used to embed
/// payload data into PNG images. Each variant contains its own configuration
/// options specific to that algorithm.
///
/// # Supported Strategies
///
/// ## LSB (Least Significant Bit)
/// The LSB strategy modifies the least significant bits of image pixel data
/// to encode payload information. This provides a good balance between
/// capacity and visual imperceptibility.
///
/// # Examples
///
/// ## Basic LSB Strategy
/// ```rust
/// use pnger::strategy::{Strategy, lsb::LSBConfig};
///
/// // Use default LSB configuration (random pattern)
/// let strategy = Strategy::default();
///
/// // Or create explicitly with linear pattern
/// let strategy = Strategy::LSB(LSBConfig::linear());
/// ```
///
/// ## Custom LSB Configuration
/// ```rust
/// use pnger::strategy::{Strategy, lsb::{LSBConfig, BitIndex}};
///
/// // Random pattern with password-derived seed
/// let strategy = Strategy::LSB(
///     LSBConfig::random()
///         .with_password("secret_key".to_string())
///         .with_bit_index(BitIndex::Bit1)
/// );
/// ```
///
/// # Security Considerations
///
/// - **Linear patterns** are faster but more detectable by statistical analysis
/// - **Random patterns** provide better security at the cost of some performance
/// - **Password-derived seeds** don't require embedding seed data in the image
/// - **Auto-generated seeds** provide maximum entropy but must be stored in the image
#[derive(Debug, Clone)]
pub enum Strategy {
    /// LSB (Least Significant Bit) steganography with configurable options.
    ///
    /// This strategy embeds data by modifying the least significant bits of
    /// image pixels. The `LSBConfig` determines the specific embedding pattern,
    /// bit positions, and security options.
    LSB(LSBConfig),
}

impl Default for Strategy {
    /// Creates a default strategy using LSB with random pattern.
    ///
    /// The default configuration uses:
    /// - Random embedding pattern for better security
    /// - Auto-generated seed (embedded in the image)
    /// - Bit index 0 (least significant bit)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::Strategy;
    ///
    /// let strategy = Strategy::default();
    /// ```
    fn default() -> Self {
        Strategy::LSB(LSBConfig::default())
    }
}
