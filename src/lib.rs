//! # PNGer - PNG Steganography Library
//!
//! PNGer is a Rust library for embedding and extracting payloads within PNG images using steganography techniques.
//! It provides both file-based and memory-based APIs for flexibility, with support for various embedding strategies
//! and payload obfuscation methods.
//!
//! ## Key Features
//!
//! - **Embedding Strategies**: For now, only LSB (Least Significant Bit) strategy is supported with linear and random patterns
//! - **Payload Obfuscation**: XOR encryption for additional security
//! - **Cross-platform**: Compatible across different architectures
//! - **Password Protection**: Derive embedding patterns from passwords
//!
//! ## Quick Start
//!
//! ### Basic embedding and extraction from files
//!
//! ```no_run
//! use pnger::{embed_payload_from_file, extract_payload_from_file};
//!
//! // Embed a payload
//! let payload = b"this is a payload";
//! let png_with_payload = embed_payload_from_file("image.png", payload)?;
//! std::fs::write("output.png", png_with_payload)?;
//!
//! // Extract the payload
//! let extracted_payload = extract_payload_from_file("output.png")?;
//! assert_eq!(extracted_payload, payload);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Basic embedding and extraction from bytes
//!
//! ```no_run
//! use pnger::{embed_payload_from_bytes, extract_payload_from_bytes};
//!
//! // Embed a payload
//! let payload = b"this is a payload";
//! let png_bytes = [137u8, 80u8, 78u8, 71u8, 13u8, 10u8, 26u8, 10u8, /* ... */];
//! let png_with_payload = embed_payload_from_bytes(&png_bytes, payload)?;
//!
//! // Extract the payload
//! let extracted_payload = extract_payload_from_bytes(&png_with_payload)?;
//! assert_eq!(&extracted_payload, payload);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Advanced Usage with Options
//!
//! ```no_run
//! use pnger::{embed_payload_from_file_with_options, EmbeddingOptions, Strategy};
//! use pnger::strategy::lsb::LSBConfig;
//! use pnger::Obfuscation;
//!
//! // Configure random pattern with password protection and XOR obfuscation
//! let strategy = Strategy::LSB(
//!     LSBConfig::random()
//!         .with_password("my_secret_password".to_string())
//!         .with_bit_index(1)
//! );
//! let options = EmbeddingOptions::new_with_obfuscation(
//!     strategy,
//!     Obfuscation::Xor { key: b"encryption_key".to_vec() }
//! );
//!
//! let payload = b"highly secure secret message";
//! let result = embed_payload_from_file_with_options("image.png", payload, options)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Embedding Strategies
//!
//! ### LSB (Least Significant Bit)
//!
//! The primary embedding strategy modifies the least significant bits of image pixels:
//!
//! - **Linear Pattern**: Sequential pixel modification (faster, less secure)
//! - **Random Pattern**: Pseudo-random pixel selection (slower, more secure)
//! - **Password Protection**: Derive random patterns from passwords
//! - **Bit Index Selection**: Choose which bit position to modify (0-7)
//!
//! ## Considerations
//!
//! - **Capacity**: 1 byte requires 8 pixels (1 bit per pixel for LSB)
//! - **Random Patterns**: Slightly slower due to PRNG operations
//!
//! ## Error Handling
//!
//! All functions return `Result<T, PngerError>` with comprehensive error types:
//!
//! - **Capacity Errors**: Payload too large for image
//! - **I/O Errors**: File system or PNG format issues
//! - **Crypto Errors**: Random number generation or password derivation failures
//! - **Format Errors**: Invalid PNG structure or corrupted data

use std::io::{BufWriter, Cursor};

// Module declarations
pub mod error;
mod io;
pub mod obfuscation;
pub mod strategy;
mod utils;

/// Wire format payload size (32-bit for cross-platform compatibility)
type PayloadSize = u32;

// Re-exports for public API
pub use crate::obfuscation::Obfuscation;
use crate::strategy::lsb::LSBEmbedder;
pub use crate::strategy::Strategy;
pub use error::PngerError;

use io::read_file;
use utils::setup_png_encoder;

/// Configuration options for payload embedding and extraction operations.
///
/// This struct combines embedding strategy selection with optional payload obfuscation
/// settings. It provides a high-level interface for configuring steganography operations
/// without needing to manage low-level implementation details.
///
/// # Examples
///
/// ## Basic Strategy Configuration
///
/// ```rust
/// use pnger::{EmbeddingOptions, Strategy};
/// use pnger::strategy::lsb::LSBConfig;
///
/// // Linear pattern (fast, less secure)
/// let options = EmbeddingOptions::new(Strategy::LSB(LSBConfig::linear()));
///
/// // Random pattern with auto-generated seed
/// let options = EmbeddingOptions::new(Strategy::LSB(LSBConfig::random()));
///
/// // Random pattern with password
/// let strategy = Strategy::LSB(
///     LSBConfig::random().with_password("secret123".to_string())
/// );
/// let options = EmbeddingOptions::new(strategy);
/// ```
///
/// ## With Obfuscation
///
/// ```rust
/// use pnger::{EmbeddingOptions, Strategy, Obfuscation};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let strategy = Strategy::LSB(LSBConfig::random());
/// let obfuscation = Obfuscation::Xor { key: b"encryption_key".to_vec() };
/// let options = EmbeddingOptions::new_with_obfuscation(strategy, obfuscation);
/// ```
#[derive(Debug, Default)]
pub struct EmbeddingOptions {
    strategy: Strategy,
    obfuscation: Option<Obfuscation>,
}

impl EmbeddingOptions {
    /// Creates new embedding options with the specified strategy.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The embedding strategy to use (currently supports LSB)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::{EmbeddingOptions, Strategy};
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// let options = EmbeddingOptions::new(Strategy::LSB(LSBConfig::linear()));
    /// ```
    pub fn new(strategy: Strategy) -> Self {
        Self {
            strategy,
            obfuscation: None,
        }
    }

    /// Creates new embedding options with strategy and obfuscation.
    ///
    /// Combines an embedding strategy with payload obfuscation for enhanced security.
    /// The payload will be obfuscated before embedding and deobfuscated after extraction.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The embedding strategy to use
    /// * `obfuscation` - The obfuscation method to apply to the payload
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::{EmbeddingOptions, Strategy, Obfuscation};
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// let strategy = Strategy::LSB(LSBConfig::random());
    /// let obfuscation = Obfuscation::Xor { key: b"secret".to_vec() };
    /// let options = EmbeddingOptions::new_with_obfuscation(strategy, obfuscation);
    /// ```
    pub fn new_with_obfuscation(strategy: Strategy, obfuscation: Obfuscation) -> Self {
        Self {
            strategy,
            obfuscation: Some(obfuscation),
        }
    }

    /// Sets the obfuscation method for these options.
    ///
    /// This method allows you to add or change the obfuscation settings after
    /// creating the embedding options.
    ///
    /// # Arguments
    ///
    /// * `obfuscation` - The obfuscation method to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::{EmbeddingOptions, Strategy, Obfuscation};
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// let mut options = EmbeddingOptions::new(Strategy::LSB(LSBConfig::linear()));
    /// options.obfuscation(Obfuscation::Xor { key: b"key".to_vec() });
    /// ```
    pub fn obfuscation(&mut self, obfuscation: Obfuscation) {
        self.obfuscation = Some(obfuscation)
    }
}

/// Extracts a payload from a PNG file using the default embedding strategy.
///
/// This function reads a PNG file and extracts any payload that was previously embedded
/// using PNGer's steganography techniques. It uses the default LSB strategy with random
/// pattern detection and automatic seed recovery.
///
/// This is the primary extraction function for most use cases, providing a simple
/// interface that handles file I/O and format detection automatically.
///
/// # Arguments
///
/// * `png_path` - Path to the PNG file containing the embedded payload
///
/// # Returns
///
/// Returns a tuple containing:
/// - `Vec<u8>` - The extracted payload data
///
/// # Examples
///
/// ```no_run
/// use pnger::extract_payload_from_file;
///
/// let payload = extract_payload_from_file("image_with_payload.png")?;
/// println!("Extracted {} bytes", payload.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The PNG file cannot be read or doesn't exist
/// - The file is not a valid PNG image
/// - No embedded payload is found in the image
/// - The embedded data is corrupted or incomplete
/// - File I/O operations fail
///
/// # Performance Notes
///
/// - File I/O operations may be slower than memory-based alternatives
/// - Consider using [`extract_payload_from_bytes`] for in-memory operations
/// - Extraction time depends on the embedding pattern used (linear vs random)
pub fn extract_payload_from_file(png_path: &str) -> Result<Vec<u8>, PngerError> {
    extract_payload_from_file_with_options(png_path, EmbeddingOptions::default())
}

/// Extracts a payload from a PNG file using custom embedding options.
///
/// This function provides advanced control over the extraction process by allowing
/// you to specify the embedding strategy and obfuscation settings that were used
/// during embedding. This is essential when non-default settings were used.
///
/// # Arguments
///
/// * `png_path` - Path to the PNG file containing the embedded payload
/// * `options` - Embedding options specifying strategy and obfuscation settings
///
/// # Returns
///
/// Returns a tuple containing:
/// - `Vec<u8>` - The extracted and deobfuscated payload data
///
/// # Examples
///
/// ## Extract with Password Protection
///
/// ```no_run
/// use pnger::{extract_payload_from_file_with_options, EmbeddingOptions, Strategy};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let strategy = Strategy::LSB(
///     LSBConfig::random().with_password("secret123".to_string())
/// );
/// let options = EmbeddingOptions::new(strategy);
///
/// let payload = extract_payload_from_file_with_options("protected_image.png", options)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Extract with Obfuscation
///
/// ```no_run
/// use pnger::{extract_payload_from_file_with_options, EmbeddingOptions, Strategy, Obfuscation};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let strategy = Strategy::LSB(LSBConfig::linear());
/// let obfuscation = Obfuscation::Xor { key: b"encryption_key".to_vec() };
/// let options = EmbeddingOptions::new_with_obfuscation(strategy, obfuscation);
///
/// let payload = extract_payload_from_file_with_options("encrypted_image.png", options )?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The PNG file cannot be read or doesn't exist
/// - The file is not a valid PNG image
/// - The extraction strategy doesn't match the embedding strategy
/// - Password or seed information is incorrect
/// - Obfuscation settings don't match those used during embedding
/// - No embedded payload is found
/// - File I/O operations fail
pub fn extract_payload_from_file_with_options(
    png_path: &str,
    options: EmbeddingOptions,
) -> Result<Vec<u8>, PngerError> {
    let png_data = read_file(png_path)?;
    extract_payload_from_bytes_with_options(&png_data, options)
}

/// Extracts a payload from PNG data in memory using the default embedding strategy.
///
/// This function operates entirely in memory, making it ideal for scenarios where
/// you already have PNG data loaded or want to avoid file I/O operations. It uses
/// the default LSB strategy with automatic pattern detection.
///
/// This is the core extraction function used internally by the file-based API and
/// provides the foundation for all extraction operations.
///
/// # Arguments
///
/// * `png_data` - Raw PNG file data as bytes
///
/// # Returns
///
/// Returns a tuple containing:
/// - `Vec<u8>` - The extracted payload data
///
/// # Examples
///
/// ```no_run
/// use pnger::extract_payload_from_bytes;
///
/// let png_data = std::fs::read("image_with_payload.png")?;
/// let payload = extract_payload_from_bytes(&png_data)?;
///
/// println!("Extracted payload: {}", String::from_utf8_lossy(&payload));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
/// # Errors
///
/// This function will return an error if:
/// - The data is not valid PNG format
/// - No embedded payload is found in the image
/// - The embedded data is corrupted or incomplete
/// - PNG decoding operations fail
///
/// # Performance Notes
///
/// - Faster than file-based operations (no I/O overhead)
/// - Memory usage scales with PNG size
/// - Consider memory constraints with very large images
pub fn extract_payload_from_bytes(png_data: &[u8]) -> Result<Vec<u8>, PngerError> {
    extract_payload_from_bytes_with_options(png_data, EmbeddingOptions::default())
}

/// Extracts a payload from PNG data using custom embedding options.
///
/// This is the most flexible extraction function, providing full control over the
/// extraction process while operating entirely in memory. It's the foundation for
/// all other extraction functions and handles advanced scenarios like password
/// protection and payload obfuscation.
///
/// # Arguments
///
/// * `png_data` - Raw PNG file data as bytes
/// * `options` - Embedding options specifying extraction strategy and deobfuscation settings
///
/// # Returns
///
/// Returns a tuple containing:
/// - `Vec<u8>` - The extracted and deobfuscated payload data
///
/// # Examples
///
/// ## Advanced Extraction with All Features
///
/// ```no_run
/// use pnger::{extract_payload_from_bytes_with_options, EmbeddingOptions, Strategy, Obfuscation};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let png_data = std::fs::read("complex_image.png")?;
///
/// // Configure extraction to match embedding settings
/// let strategy = Strategy::LSB(
///     LSBConfig::random()
///         .with_password("my_secret_password".to_string())
///         .with_bit_index(2)
/// );
/// let obfuscation = Obfuscation::Xor { key: b"encryption_key".to_vec() };
/// let options = EmbeddingOptions::new_with_obfuscation(strategy, obfuscation);
///
/// let payload = extract_payload_from_bytes_with_options(&png_data, options)?;
///
/// // The payload is now decrypted and ready to use
/// println!("Secret message: {}", String::from_utf8_lossy(&payload));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Batch Processing
///
/// ```rust
/// use pnger::{extract_payload_from_bytes_with_options, EmbeddingOptions, Strategy};
/// use pnger::strategy::lsb::LSBConfig;
///
/// fn process_images(images: Vec<Vec<u8>>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
///     let mut results = Vec::new();
///     for png_data in images {
///         let strategy = Strategy::LSB(LSBConfig::linear());
///         let options = EmbeddingOptions::new(strategy);
///         let payload = extract_payload_from_bytes_with_options(&png_data, options)?;
///         results.push(String::from_utf8(payload)?);
///     }
///     Ok(results)
/// }
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The data is not valid PNG format
/// - The extraction strategy doesn't match the embedding strategy
/// - Password or cryptographic settings are incorrect
/// - Obfuscation key doesn't match the one used during embedding
/// - No embedded payload is found or data is corrupted
/// - PNG decoding operations fail
///
/// # Security Considerations
///
/// - Always use the same password that was used during embedding
/// - Obfuscation keys must match exactly (case-sensitive)
/// - Failed extraction may indicate wrong credentials or corrupted data
/// - Consider implementing retry logic with different parameters if needed
pub fn extract_payload_from_bytes_with_options(
    png_data: &[u8],
    options: EmbeddingOptions,
) -> Result<Vec<u8>, PngerError> {
    let (mut reader, _) = decode_png_info(png_data)?;
    let mut image_data = read_image_data(&mut reader)?;

    let payload_data = match options.strategy {
        Strategy::LSB(lsb_config) => LSBEmbedder::extract(&mut image_data, &lsb_config)?.payload,
    };

    let final_payload = match options.obfuscation {
        Some(obfuscation) => obfuscation::deobfuscate_payload(&payload_data, obfuscation),
        None => payload_data,
    };

    Ok(final_payload)
}

// ===== Embedding methods =====

/// Embeds a payload into a PNG file using the default embedding strategy.
///
/// This function takes a PNG file path and payload data, then embeds the payload
/// into the image using the default LSB (Least Significant Bit) strategy with
/// random pattern embedding and auto-generated seed.
///
/// This is the primary embedding function for most use cases, providing a simple
/// interface that handles file I/O operations automatically while maintaining good security.
///
/// # Arguments
///
/// * `png_path` - Path to the source PNG file to modify
/// * `payload_data` - Raw bytes to embed in the image
///
/// # Returns
///
/// Returns the modified PNG data as bytes on success, or a [`PngerError`] on failure.
/// The returned data can be written directly to a file to create the steganographic image.
///
/// # Examples
///
/// ```no_run
/// use pnger::embed_payload_from_file;
///
/// let payload = b"This is my secret message";
/// let result = embed_payload_from_file("source.png", payload)?;
/// std::fs::write("output_with_payload.png", result)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Embedding Text Messages
///
/// ```no_run
/// use pnger::embed_payload_from_file;
///
/// let secret_message = "Meet me at midnight";
/// let payload = secret_message.as_bytes();
/// let result = embed_payload_from_file("cover_image.png", payload)?;
///
/// // Save the steganographic image
/// std::fs::write("stego_image.png", result)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The PNG file cannot be read or doesn't exist
/// - The file is not a valid PNG image
/// - The payload is too large for the image capacity
/// - The image has insufficient pixels for the payload size
/// - File I/O operations fail
/// - PNG encoding/decoding operations fail
///
/// # Capacity Considerations
///
/// The embedding capacity depends on the image size and strategy:
/// - **LSB Strategy**: Requires 8 pixels per payload byte (1 bit per pixel)
/// - **Header Overhead**: Additional pixels needed for metadata storage
/// - **Seed Storage**: Random patterns may embed seed data, consuming more pixels
///
/// For a 1000x1000 pixel image, you can typically embed around 125KB of payload data.
///
/// # Performance Notes
///
/// - File I/O operations add overhead compared to memory-based functions
/// - Random patterns are slightly slower than linear due to PRNG operations
/// - Consider using [`embed_payload_from_bytes`] for better performance in batch operations
pub fn embed_payload_from_file(png_path: &str, payload_data: &[u8]) -> Result<Vec<u8>, PngerError> {
    embed_payload_from_file_with_options(png_path, payload_data, EmbeddingOptions::default())
}

/// Embeds a payload into a PNG file using custom embedding options.
///
/// This function provides advanced control over the embedding process, allowing you
/// to specify the embedding strategy, obfuscation settings, and other parameters.
/// It's ideal for scenarios requiring specific security or performance characteristics.
///
/// # Arguments
///
/// * `png_path` - Path to the source PNG file to modify
/// * `payload_data` - Raw bytes to embed in the image  
/// * `options` - Embedding options specifying strategy and obfuscation settings
///
/// # Returns
///
/// Returns the modified PNG data as bytes, ready to be written to a file.
///
/// # Examples
///
/// ## Linear Pattern (Fast, Less Secure)
///
/// ```no_run
/// use pnger::{embed_payload_from_file_with_options, EmbeddingOptions, Strategy};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let payload = b"Fast embedding example";
/// let strategy = Strategy::LSB(LSBConfig::linear());
/// let options = EmbeddingOptions::new(strategy);
///
/// let result = embed_payload_from_file_with_options("image.png", payload, options)?;
/// std::fs::write("fast_output.png", result)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Password-Protected Random Pattern
///
/// ```no_run
/// use pnger::{embed_payload_from_file_with_options, EmbeddingOptions, Strategy};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let payload = b"Highly secure secret data";
/// let strategy = Strategy::LSB(
///     LSBConfig::random()
///         .with_password("my_secret_password".to_string())
///         .with_bit_index(2)  // Use bit position 2 instead of 0
/// );
/// let options = EmbeddingOptions::new(strategy);
///
/// let result = embed_payload_from_file_with_options("image.png", payload, options)?;
/// std::fs::write("secure_output.png", result)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## With XOR Obfuscation
///
/// ```no_run
/// use pnger::{embed_payload_from_file_with_options, EmbeddingOptions, Strategy, Obfuscation};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let payload = b"Double-encrypted secret";
/// let strategy = Strategy::LSB(LSBConfig::random());
/// let obfuscation = Obfuscation::Xor { key: b"encryption_key".to_vec() };
/// let options = EmbeddingOptions::new_with_obfuscation(strategy, obfuscation);
///
/// let result = embed_payload_from_file_with_options("image.png", payload, options)?;
/// std::fs::write("encrypted_output.png", result)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The PNG file cannot be read or doesn't exist
/// - The file is not a valid PNG image
/// - The payload is too large for the image capacity
/// - Invalid embedding parameters (e.g., bit_index > 7)
/// - Cryptographic operations fail (password derivation, seed generation)
/// - File I/O or PNG processing operations fail
///
/// # Strategy Considerations
///
/// ## Linear vs Random Patterns
///
/// - **Linear**: Faster embedding, sequential pixel modification, easier to detect
/// - **Random**: Slower embedding, scattered pixel modification, harder to detect
///
/// ## Password Protection
///
/// - Uses Argon2 for secure password-to-seed derivation
/// - No seed data is embedded in the image (smaller overhead)
/// - Must remember the exact password for extraction
///
/// ## Bit Index Selection
///
/// - Index 0 (LSB): Most common, good invisibility vs capacity trade-off
/// - Higher indices: Less capacity, potentially more visible, but less predictable
pub fn embed_payload_from_file_with_options(
    png_path: &str,
    payload_data: &[u8],
    options: EmbeddingOptions,
) -> Result<Vec<u8>, PngerError> {
    let png_data = read_file(png_path)?;
    embed_payload_from_bytes_with_options(&png_data, payload_data, options)
}

/// Embeds a payload into PNG data in memory using the default embedding strategy.
///
/// This function operates entirely in memory, making it ideal for scenarios where
/// you already have PNG data loaded or want to avoid file I/O operations. It uses
/// the default LSB strategy with random pattern and auto-generated seed for good
/// security with minimal configuration.
///
/// This is the core embedding function used internally by the file-based API and
/// provides the foundation for all embedding operations.
///
/// # Arguments
///
/// * `png_data` - Raw PNG file data as bytes
/// * `payload_data` - Raw bytes to embed in the image
///
/// # Returns
///
/// Returns the modified PNG data as bytes, ready for use or storage.
///
/// # Examples
///
/// ```no_run
/// use pnger::embed_payload_from_bytes;
///
/// let png_bytes = [137u8, 80u8, 78u8, 71u8, 13u8, 10u8, 26u8, 10u8, /* ... */];
/// let payload = b"message to hide";
///
/// let result = embed_payload_from_bytes(&png_bytes, payload)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The data is not valid PNG format
/// - The payload is too large for the image capacity
/// - PNG decoding or encoding operations fail
/// - Memory allocation fails
///
/// # Performance Notes
///
/// - Faster than file-based operations (no I/O overhead)
/// - Memory usage scales with PNG size (typically 3-4x the PNG file size during processing)
/// - Ideal for web applications and batch processing
/// - Consider memory constraints with very large images or many concurrent operations
///
/// # Capacity Guidelines
///
/// For LSB embedding, the theoretical capacity is:
/// ```text
/// Capacity ≈ (Image Width × Image Height × Channels) / 8 bytes
/// ```
///
/// Practical capacity is lower due to header overhead:
/// - Small images (< 100KB): ~60-80% of theoretical capacity
/// - Large images (> 1MB): ~90-95% of theoretical capacity
pub fn embed_payload_from_bytes(
    png_data: &[u8],
    payload_data: &[u8],
) -> Result<Vec<u8>, PngerError> {
    embed_payload_from_bytes_with_options(png_data, payload_data, EmbeddingOptions::default())
}

/// Embeds a payload into PNG data using custom embedding options.
///
/// This is the most flexible embedding function, providing full control over the
/// embedding process while operating entirely in memory. It handles all advanced
/// scenarios including custom strategies, password protection, obfuscation, and
/// fine-tuned embedding parameters.
///
/// # Arguments
///
/// * `png_data` - Raw PNG file data as bytes
/// * `payload_data` - Raw bytes to embed in the image
/// * `options` - Embedding options specifying strategy and obfuscation settings
///
/// # Returns
///
/// Returns the modified PNG data as bytes with the embedded payload.
///
/// # Examples
///
/// ## Maximum Security Configuration
///
/// ```no_run
/// use pnger::{embed_payload_from_bytes_with_options, EmbeddingOptions, Strategy, Obfuscation};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let png_bytes = [137u8, 80u8, 78u8, 71u8, 13u8, 10u8, 26u8, 10u8, /* ... */];
/// let payload = b"super_secret_payload";
///
/// // Configure maximum security
/// let strategy = Strategy::LSB(
///     LSBConfig::random()
///         .with_password("ultra_secure_password_123".to_string())
///         .with_bit_index(2)  // Use less predictable bit position
/// );
/// let obfuscation = Obfuscation::Xor {
///     key: b"additional_encryption_layer".to_vec()
/// };
/// let options = EmbeddingOptions::new_with_obfuscation(strategy, obfuscation);
///
/// let png_with_secret_payload = embed_payload_from_bytes_with_options(&png_bytes, payload, options)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Performance-Optimized Configuration
///
/// ```no_run
/// use pnger::{embed_payload_from_bytes_with_options, EmbeddingOptions, Strategy};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let png_data = std::fs::read("source.png")?;
/// let payload = b"Fast embedding for batch processing";
///
/// // Configure for speed
/// let strategy = Strategy::LSB(LSBConfig::linear());  // Linear is fastest
/// let options = EmbeddingOptions::new(strategy);
///
/// let result = embed_payload_from_bytes_with_options(&png_data, payload, options)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Custom Bit Index for Multiple Payloads
///
/// ```no_run
/// use pnger::{embed_payload_from_bytes_with_options, EmbeddingOptions, Strategy};
/// use pnger::strategy::lsb::LSBConfig;
///
/// let mut png_data = std::fs::read("source.png")?;
///
/// // Embed first payload in bit 0
/// let strategy1 = Strategy::LSB(LSBConfig::linear().with_bit_index(0));
/// let options1 = EmbeddingOptions::new(strategy1);
/// png_data = embed_payload_from_bytes_with_options(&png_data, b"First payload", options1)?;
///
/// // Embed second payload in bit 1 (same image)
/// let strategy2 = Strategy::LSB(LSBConfig::linear().with_bit_index(1));
/// let options2 = EmbeddingOptions::new(strategy2);
/// png_data = embed_payload_from_bytes_with_options(&png_data, b"Second payload", options2)?;
///
/// std::fs::write("multi_payload.png", png_data)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The data is not valid PNG format
/// - The payload is too large for the image capacity
/// - Invalid embedding parameters (bit_index > 7, empty password, etc.)
/// - Cryptographic operations fail (PRNG, password derivation)
/// - PNG processing operations fail
/// - Memory allocation fails
///
/// # Advanced Configuration Guide
///
/// ## Embedding Strategies
///
/// ### Linear Pattern
/// - **Use case**: High-speed batch processing, non-critical data
/// - **Security**: Low (predictable pattern)
/// - **Performance**: Highest
/// - **Detection resistance**: Low
///
/// ### Random Pattern (Auto Seed)
/// - **Use case**: Good security with convenience
/// - **Security**: High (unpredictable pattern)
/// - **Performance**: Medium
/// - **Detection resistance**: High
/// - **Note**: Seed is embedded in image (slight capacity overhead)
///
/// ### Random Pattern (Password)
/// - **Use case**: Maximum security for sensitive data
/// - **Security**: Highest (password-derived seed)
/// - **Performance**: Medium
/// - **Detection resistance**: Highest
/// - **Note**: No seed embedded (maximum capacity)
///
/// ## Bit Index Selection
///
/// - **Index 0 (LSB)**: Standard choice, best invisibility/capacity ratio
/// - **Index 1-2**: Good alternative indices, slightly more visible
/// - **Index 3-7**: Higher visibility, use only for multiple payload scenarios
///
/// ## Obfuscation Methods
///
/// ### XOR Encryption
/// - **Overhead**: Minimal (no size increase)
/// - **Security**: Moderate (depends on key strength)
/// - **Performance**: Excellent (simple bitwise operations)
/// - **Use case**: Additional security layer, key-based access control
pub fn embed_payload_from_bytes_with_options(
    png_data: &[u8],
    payload_data: &[u8],
    options: EmbeddingOptions,
) -> Result<Vec<u8>, PngerError> {
    let (mut reader, info) = decode_png_info(png_data)?;
    let mut image_data = read_image_data(&mut reader)?;
    let payload_data = match options.obfuscation {
        Some(obfuscation) => &obfuscation::obfuscate_payload(payload_data, obfuscation),
        _ => payload_data,
    };

    match options.strategy {
        Strategy::LSB(lsb_config) => {
            LSBEmbedder::embed(&mut image_data, payload_data, &lsb_config)?;
        }
    }
    encode_png_with_data(&info, &image_data)
}

type DecodedPngInfo<'a> = Result<(png::Reader<Cursor<&'a [u8]>>, png::Info<'a>), PngerError>;

/// Decodes PNG data and extracts format information.
///
/// This internal function handles the initial PNG decoding step, creating a reader
/// and extracting metadata needed for subsequent embedding or extraction operations.
///
/// # Arguments
///
/// * `png_data` - Raw PNG file data as bytes
///
/// # Returns
///
/// Returns a tuple containing the PNG reader and format information, or an error
/// if the PNG data is invalid or corrupted.
///
/// # Errors
///
/// This function will return an error if:
/// - The data is not valid PNG format
/// - PNG headers are corrupted or malformed
/// - Unsupported PNG variants or extensions
fn decode_png_info(png_data: &[u8]) -> DecodedPngInfo {
    let decoder = png::Decoder::new(Cursor::new(png_data));
    let reader = decoder.read_info()?;
    let info = reader.info().clone();
    Ok((reader, info))
}

/// Reads raw pixel data from a PNG reader into memory.
///
/// This function extracts the raw image pixel data that will be used for
/// steganographic operations. The data is returned in the format expected
/// by the embedding and extraction algorithms.
///
/// # Arguments
///
/// * `reader` - PNG reader positioned after header parsing
///
/// # Returns
///
/// Returns the raw image data as a byte vector, or an error if reading fails.
///
/// # Errors
///
/// This function will return an error if:
/// - PNG data is corrupted or incomplete
/// - Memory allocation fails
/// - PNG decompression fails
fn read_image_data(reader: &mut png::Reader<Cursor<&[u8]>>) -> Result<Vec<u8>, PngerError> {
    let mut image_data = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut image_data)?;
    Ok(image_data)
}

/// Encodes image data back into PNG format.
///
/// This function takes modified image data (after embedding operations) and
/// reconstructs a valid PNG file with the same format characteristics as the
/// original image.
///
/// # Arguments
///
/// * `info` - PNG format information from the original image
/// * `image_data` - Modified pixel data containing embedded payload
///
/// # Returns
///
/// Returns the complete PNG file as bytes, ready for storage or transmission.
///
/// # Errors
///
/// This function will return an error if:
/// - PNG encoding operations fail
/// - Image data size doesn't match expected dimensions
/// - Memory allocation or buffer operations fail
fn encode_png_with_data(info: &png::Info, image_data: &[u8]) -> Result<Vec<u8>, PngerError> {
    let mut writer_buffer = BufWriter::new(Vec::new());
    let encoder = setup_png_encoder(info, &mut writer_buffer)?;

    let mut writer = encoder.write_header()?;
    writer.write_image_data(image_data)?;
    writer.finish()?;

    writer_buffer.into_inner().map_err(|e| PngerError::IoError {
        message: format!("Failed to extract buffer: {}", e),
    })
}
