//! # Error types for PNGer Steganography Operations
//!
//! This module defines comprehensive error types for all PNGer operations, providing
//! detailed information about failures during embedding and extraction processes.
//! The error types are designed to help users diagnose issues and implement appropriate
//! error handling strategies.
//!
//! ## Error Hierarchy
//!
//! PNGer errors are organized into logical categories:
//!
//! - **Capacity Errors**: Issues related to image size vs payload size
//! - **Format Errors**: PNG parsing, encoding, or steganographic format issues  
//! - **I/O Errors**: File system and data transfer problems
//! - **Cryptographic Errors**: Password, encryption, and random number generation failures
//! - **Processing Errors**: Payload handling and operation mode issues
//!
//! ## Error Handling Patterns
//!
//! ### Basic Pattern Matching
//!
//! ```rust
//! use pnger::{embed_payload_from_file, PngerError};
//!
//! match embed_payload_from_file("image.png", b"secret") {
//!     Ok(result) => println!("Success!"),
//!     Err(PngerError::PayloadTooLarge) => println!("Payload too large"),
//!     Err(PngerError::FileIo(_)) => println!("File access error"),
//!     Err(err) => println!("Other error: {}", err),
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Error Recovery Strategies
//!
//! ```rust
//! use pnger::{embed_payload_from_file, PngerError};
//!
//! fn embed_with_retry(image_path: &str, payload: &[u8]) -> Result<Vec<u8>, String> {
//!     match embed_payload_from_file(image_path, payload) {
//!         Ok(result) => Ok(result),
//!         Err(PngerError::PayloadTooLarge) => {
//!             // Try with compression or smaller payload
//!             Err("Payload too large, try compressing first".to_string())
//!         }
//!         Err(PngerError::FileIo(_)) => {
//!             // Retry with different file path or check permissions
//!             Err("File access failed, check path and permissions".to_string())
//!         }
//!         Err(err) => Err(format!("Embedding failed: {}", err)),
//!     }
//! }
//! ```

use std::io;
use thiserror::Error;

/// Comprehensive error type for all PNGer steganography operations.
///
/// This enum covers all possible failure modes in the PNGer library, from basic
/// I/O errors to advanced cryptographic failures. Each variant provides specific
/// context about the nature of the failure to enable proper error handling.
///
/// # Error Categories
///
/// ## Capacity Errors
/// - [`PayloadTooLarge`](PngerError::PayloadTooLarge): Payload exceeds image capacity
/// - [`InsufficientCapacity`](PngerError::InsufficientCapacity): Image too small for payload
///
/// ## Format Errors  
/// - [`PngDecodingError`](PngerError::PngDecodingError): Invalid or corrupted PNG data
/// - [`PngEncodingError`](PngerError::PngEncodingError): PNG reconstruction failed
/// - [`InvalidFormat`](PngerError::InvalidFormat): Malformed steganographic data
///
/// ## I/O Errors
/// - [`FileIo`](PngerError::FileIo): File system operations failed
/// - [`IoError`](PngerError::IoError): General I/O operations failed
///
/// ## Cryptographic Errors
/// - [`CryptoError`](PngerError::CryptoError): Password derivation or encryption failed
/// - [`RandomGenerationFailed`](PngerError::RandomGenerationFailed): PRNG operations failed
/// - [`InvalidSeedLength`](PngerError::InvalidSeedLength): Invalid cryptographic seed
/// - [`InvalidSaltLength`](PngerError::InvalidSaltLength): Invalid salt for key derivation
///
/// ## Processing Errors
/// - [`PayloadError`](PngerError::PayloadError): Payload processing failed
/// - [`UnsupportedMode`](PngerError::UnsupportedMode): Unsupported operation mode
///
/// # Examples
///
/// ## Basic Error Handling
///
/// ```rust
/// use pnger::{embed_payload_from_file, PngerError};
///
/// match embed_payload_from_file("image.png", b"secret") {
///     Ok(result) => {
///         std::fs::write("output.png", result)?;
///         println!("Embedding successful!");
///     }
///     Err(PngerError::PayloadTooLarge) => {
///         eprintln!("Error: The secret message is too large for this image.");
///         eprintln!("Try using a larger image or smaller payload.");
///     }
///     Err(PngerError::FileIo(io_err)) => {
///         eprintln!("File error: {}", io_err);
///     }
///     Err(other) => {
///         eprintln!("Other error: {}", other);
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Comprehensive Error Handling
///
/// ```rust
/// use pnger::{extract_payload_from_file_with_options, EmbeddingOptions, Strategy, PngerError};
/// use pnger::strategy::lsb::LSBConfig;
///
/// fn extract_with_fallback(file_path: &str, password: &str) -> Result<Vec<u8>, String> {
///     let strategy = Strategy::LSB(LSBConfig::random().with_password(password.to_string()));
///     let options = EmbeddingOptions::new(strategy);
///     
///     match extract_payload_from_file_with_options(file_path, options) {
///         Ok(payload) => Ok(payload),
///         Err(PngerError::CryptoError(_)) => {
///             Err("Incorrect password or corrupted cryptographic data".to_string())
///         }
///         Err(PngerError::InvalidFormat(msg)) => {
///             Err(format!("No valid payload found: {}", msg))
///         }
///         Err(PngerError::PngDecodingError(_)) => {
///             Err("File is not a valid PNG image".to_string())
///         }
///         Err(err) => {
///             Err(format!("Extraction failed: {}", err))
///         }
///     }
/// }
/// ```
#[derive(Error, Debug)]
pub enum PngerError {
    /// The payload is too large to fit in the specified image.
    ///
    /// This error occurs when the payload size exceeds the theoretical capacity
    /// of the image for the chosen embedding strategy. The capacity depends on
    /// image dimensions, color depth, and embedding parameters.
    ///
    /// ## Common Causes
    /// - Payload size exceeds image pixel count (for LSB embedding)
    /// - Very small images with large payloads
    /// - Multiple payloads embedded in the same bit positions
    ///
    /// ## Solutions
    /// - Use a larger image with more pixels
    /// - Compress the payload before embedding
    /// - Split the payload across multiple images
    /// - Use different bit indices for multiple payloads
    #[error("Payload is too large for the image")]
    PayloadTooLarge,

    /// The image has insufficient capacity for the payload and metadata.
    ///
    /// This error is more specific than `PayloadTooLarge` and indicates that
    /// while the payload might theoretically fit, there's insufficient space
    /// when accounting for headers, metadata, and embedding overhead.
    ///
    /// ## Common Causes
    /// - Header space requirements exceed available pixels
    /// - Random pattern seed storage requirements
    /// - Insufficient margin for embedding reliability
    ///
    /// ## Solutions
    /// - Use a slightly larger image
    /// - Switch to linear patterns (less overhead)
    /// - Use password-derived seeds (no embedded seed data)
    #[error("Insufficient capacity in image for payload")]
    InsufficientCapacity,

    /// The specified embedding mode or strategy is not supported.
    ///
    /// This error indicates that the requested operation mode is not implemented
    /// or is invalid for the current context.
    ///
    /// ## Common Causes
    /// - Invalid embedding strategy parameters
    /// - Unsupported PNG color modes
    /// - Invalid bit index values (> 7)
    #[error("Unsupported embedding mode")]
    UnsupportedMode,

    /// A general I/O operation failed with additional context.
    ///
    /// This error provides detailed information about I/O failures that don't
    /// fit into more specific categories.
    ///
    /// ## Common Causes
    /// - Buffer operations failed
    /// - Network I/O errors in distributed scenarios
    /// - Memory mapping failures
    #[error("I/O error: {message}")]
    IoError {
        /// Detailed description of the I/O failure
        message: String,
    },

    /// PNG decoding operation failed.
    ///
    /// This error occurs when the input data cannot be decoded as a valid PNG image.
    /// It wraps the underlying PNG library error for detailed diagnostics.
    ///
    /// ## Common Causes
    /// - File is not a PNG image
    /// - PNG data is corrupted or truncated
    /// - Unsupported PNG variants or extensions
    /// - Invalid PNG headers or chunk data
    #[error("PNG decoding error: {0}")]
    PngDecodingError(#[from] png::DecodingError),

    /// PNG encoding operation failed.
    ///
    /// This error occurs when modified image data cannot be encoded back into
    /// valid PNG format, typically after embedding operations.
    ///
    /// ## Common Causes
    /// - Invalid image dimensions after modification
    /// - Corrupted pixel data from embedding operations
    /// - Memory allocation failures during encoding
    #[error("PNG encoding error: {0}")]
    PngEncodingError(#[from] png::EncodingError),

    /// Payload processing operation failed.
    ///
    /// This error indicates that payload-specific operations like obfuscation,
    /// compression, or format conversion failed.
    ///
    /// ## Common Causes
    /// - Obfuscation key errors
    /// - Payload format conversion failures
    /// - Data integrity check failures
    #[error("Payload processing error: {message}")]
    PayloadError {
        /// Specific description of the payload processing failure
        message: String,
    },

    /// File system I/O operation failed.
    ///
    /// This error wraps standard file system errors from operations like
    /// reading PNG files or writing output data.
    ///
    /// ## Common Causes
    /// - File not found or access denied
    /// - Disk space exhausted
    /// - Network file system errors
    /// - Permission issues
    #[error("File I/O failed")]
    FileIo(#[from] io::Error),

    /// Cryptographic operation failed.
    ///
    /// This error occurs when cryptographic operations like password derivation,
    /// seed generation, or encryption/decryption fail.
    ///
    /// ## Common Causes
    /// - Password derivation using Argon2 failed
    /// - Invalid cryptographic parameters
    /// - Insufficient entropy for key generation
    /// - Hardware security module errors
    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),

    /// Random number generation failed.
    ///
    /// This error indicates that the system's random number generator is
    /// unavailable or failed to produce the required random data.
    ///
    /// ## Common Causes
    /// - System PRNG is not available
    /// - Insufficient entropy in the system
    /// - Hardware RNG failures
    /// - Virtualized environment restrictions
    #[error("Random number generation failed")]
    RandomGenerationFailed,

    /// The provided cryptographic seed has invalid length.
    ///
    /// This error occurs when manually provided seeds don't meet the required
    /// length specifications for the cryptographic operations.
    ///
    /// ## Expected Length
    /// - Seeds must be exactly 32 bytes (256 bits)
    ///
    /// ## Common Causes
    /// - Manually provided seed is too short or too long
    /// - Seed derivation produced wrong length output
    /// - Corrupted seed data during transmission
    #[error("Invalid seed length")]
    InvalidSeedLength,

    /// The provided salt has invalid length for key derivation.
    ///
    /// This error occurs when password-based key derivation operations receive
    /// salt data that doesn't meet the required specifications.
    ///
    /// ## Expected Length
    /// - Salts should typically be 16-32 bytes
    ///
    /// ## Common Causes
    /// - Manually provided salt is wrong length
    /// - Salt generation failed
    /// - Configuration mismatch between embedding and extraction
    #[error("Invalid salt length")]
    InvalidSaltLength,

    /// The file format is invalid or corrupted.
    ///
    /// This error provides detailed information about format-specific issues
    /// that prevent successful processing.
    ///
    /// ## Common Causes
    /// - Steganographic headers are corrupted
    /// - No embedded payload found in the image
    /// - Incompatible embedding format versions
    /// - Data corruption during storage or transmission
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
}
