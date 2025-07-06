//! # Payload Obfuscation for Enhanced Security
//!
//! This module provides payload obfuscation capabilities that add an additional layer
//! of security to steganographic operations. Obfuscation transforms the payload data
//! before embedding, making it harder to detect and analyze even if the steganographic
//! data is discovered.
//!
//! For now, only XOR encryption is supported. More encryption methods will be added in the future.
//!
//! ## XOR Encryption
//!
//! - **Simple**: Same key used for encryption and decryption
//! - **Minimal Overhead**: No size increase in payload data
//!
//! ## Usage Examples
//!
//! ### Embedding with XOR encryption
//!
//! ```no_run
//! use pnger::{embed_payload_from_bytes_with_options, EmbeddingOptions, Strategy, Obfuscation};
//! use pnger::strategy::lsb::LSBConfig;
//!
//! let png_data = std::fs::read("image.png")?;
//! let payload = b"the payload";
//!
//! // Configure strategy with obfuscation
//! let strategy = Strategy::LSB(LSBConfig::random());
//! let obfuscation = Obfuscation::Xor { key: b"secure_key_123".to_vec() };
//! let options = EmbeddingOptions::new_with_obfuscation(strategy, obfuscation);
//!
//! let result = embed_payload_from_bytes_with_options(&png_data, payload, options)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

/// Enumeration of available payload obfuscation methods.
#[derive(Debug, Clone)]
pub enum Obfuscation {
    /// XOR-based obfuscation using a repeating key.
    ///
    /// This method applies XOR operations between payload bytes and a cycling
    /// encryption key. The same key should be used for both obfuscation and deobfuscation.
    ///
    /// **Advantages:**
    /// - Fast encryption/decryption
    /// - Zero size overhead (payload size unchanged)
    /// - Reversibility
    ///
    /// **Security Notes:**
    /// - Security depends entirely on key secrecy
    /// - Vulnerable to known-plaintext attacks if key is reused
    /// - To increase security, should be combined with strong steganographic patterns
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::Obfuscation;
    ///
    /// let simple = Obfuscation::Xor { key: b"key123".to_vec() };
    /// ```
    Xor {
        /// The encryption key used for XOR operations.
        ///
        /// This key will be cycled through repeatedly to obfuscate payload data.
        /// The same key must be used for both obfuscation and deobfuscation.
        key: Vec<u8>,
    },
}

/// Obfuscates payload data using the specified obfuscation method.
///
/// This function transforms the input payload data according to the chosen
/// obfuscation algorithm, making it suitable for steganographic embedding
/// with enhanced security.
///
/// # Arguments
///
/// * `payload_data` - The raw payload data to obfuscate
/// * `obfuscation` - The obfuscation method and configuration to use
///
/// # Returns
///
/// Returns the obfuscated payload data. The output size is identical to the
/// input size for all current obfuscation methods.
pub(crate) fn obfuscate_payload(payload_data: &[u8], obfuscation: Obfuscation) -> Vec<u8> {
    match obfuscation {
        Obfuscation::Xor { key } => xor_payload(payload_data, &key),
    }
}

/// Deobfuscates payload data using the specified obfuscation method.
///
/// This function reverses the obfuscation process, recovering the original
/// payload data from its obfuscated form. The same obfuscation configuration
/// used for obfuscation must be provided for successful recovery.
///
/// # Arguments
///
/// * `payload_data` - The obfuscated payload data to recover
/// * `obfuscation` - The obfuscation method and configuration (must match the one used for obfuscation)
///
/// # Returns
///
/// Returns the original payload data. For XOR obfuscation, this is guaranteed
/// to be identical to the original input.
pub(crate) fn deobfuscate_payload(payload_data: &[u8], obfuscation: Obfuscation) -> Vec<u8> {
    match obfuscation {
        Obfuscation::Xor { key } => xor_payload(payload_data, &key),
    }
}

/// Performs XOR encryption/decryption of payload data with a cycling key.
///
/// This is the core XOR implementation used by both obfuscation and deobfuscation
/// operations. XOR is symmetric, so the same function serves both purposes.
///
/// # Arguments
///
/// * `payload_data` - The data to encrypt/decrypt
/// * `key` - The encryption key (will cycle if shorter than payload)
///
/// # Returns
///
/// Returns the XOR-transformed data.
pub(crate) fn xor_payload(payload_data: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return payload_data.to_vec();
    }
    payload_data
        .iter()
        .zip(key.iter().cycle())
        .map(|(byte, key_byte)| byte ^ key_byte)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_payload_basic() {
        let payload = b"Hello, World!";
        let key = b"key";
        let result = xor_payload(payload, key);

        // XOR should be reversible
        let decrypted = xor_payload(&result, key);
        assert_eq!(decrypted, payload);
    }

    #[test]
    fn test_xor_payload_empty_payload() {
        let payload = b"";
        let key = b"key";
        let result = xor_payload(payload, key);

        assert_eq!(result, Vec::<u8>::new());
    }

    #[test]
    fn test_xor_payload_empty_key() {
        let payload = b"payload";
        let key = b"";
        let result = xor_payload(payload, key);

        assert_eq!(&result, payload);
    }

    #[test]
    fn test_xor_payload_single_byte_key() {
        let payload = b"test";
        let key = b"x";
        let result = xor_payload(payload, key);

        // Each byte should be XORed with the same key byte
        let expected: Vec<u8> = payload.iter().map(|&b| b ^ b'x').collect();
        assert_eq!(result, expected);

        // Verify reversibility
        let decrypted = xor_payload(&result, key);
        assert_eq!(decrypted, payload);
    }

    #[test]
    fn test_xor_payload_key_longer_than_payload() {
        let payload = b"hi";
        let key = b"verylongkey";
        let result = xor_payload(payload, key);

        // Should only use first 2 bytes of key
        let expected = vec![b'h' ^ b'v', b'i' ^ b'e'];
        assert_eq!(result, expected);

        // Verify reversibility
        let decrypted = xor_payload(&result, key);
        assert_eq!(decrypted, payload);
    }

    #[test]
    fn test_xor_payload_key_shorter_than_payload() {
        let payload = b"verylongpayload";
        let key = b"ab";
        let result = xor_payload(payload, key);

        // Key should cycle: a, b, a, b, a, b, ...
        let expected: Vec<u8> = payload
            .iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key[i % key.len()])
            .collect();
        assert_eq!(result, expected);

        // Verify reversibility
        let decrypted = xor_payload(&result, key);
        assert_eq!(decrypted, payload);
    }

    #[test]
    fn test_xor_payload_zero_key() {
        let payload = b"test";
        let key = b"\x00\x00\x00\x00";
        let result = xor_payload(payload, key);

        // XOR with zero should return original payload
        assert_eq!(result, payload);
    }

    #[test]
    fn test_xor_payload_binary_data() {
        let payload = vec![0x00, 0xFF, 0x55, 0xAA, 0x33];
        let key = vec![0x11, 0x22];
        let result = xor_payload(&payload, &key);

        let expected = vec![
            0x11,        // 0x11
            0xFF ^ 0x22, // 0xDD
            0x55 ^ 0x11, // 0x44
            0xAA ^ 0x22, // 0x88
            0x33 ^ 0x11, // 0x22
        ];
        assert_eq!(result, expected);

        // Verify reversibility
        let decrypted = xor_payload(&result, &key);
        assert_eq!(decrypted, payload);
    }

    #[test]
    fn test_xor_payload_same_data_and_key() {
        let data = b"same";
        let result = xor_payload(data, data);

        // XOR with itself should produce all zeros
        let expected = vec![0u8; data.len()];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_xor_payload_large_data() {
        let payload = vec![0x42; 1000]; // 1KB of 0x42
        let key = b"secretkey";
        let result = xor_payload(&payload, key);

        // Verify pattern repeats correctly
        for (i, &byte) in result.iter().enumerate() {
            let expected = 0x42 ^ key[i % key.len()];
            assert_eq!(byte, expected);
        }

        // Verify reversibility
        let decrypted = xor_payload(&result, key);
        assert_eq!(decrypted, payload);
    }
}
