use std::io;
use thiserror::Error;

/// Error types for PNGer operations
#[derive(Error, Debug)]
pub enum PngerError {
    #[error("Payload is too large for the image")]
    PayloadTooLarge,

    #[error("Insufficient capacity in image for payload")]
    InsufficientCapacity,

    #[error("Unsupported embedding mode")]
    UnsupportedMode,

    #[error("I/O error: {message}")]
    IoError { message: String },

    #[error("PNG decoding error: {0}")]
    PngDecodingError(#[from] png::DecodingError),

    #[error("PNG encoding error: {0}")]
    PngEncodingError(#[from] png::EncodingError),

    #[error("Payload processing error: {message}")]
    PayloadError { message: String },

    #[error("File I/O failed")]
    FileIo(#[from] io::Error),
}