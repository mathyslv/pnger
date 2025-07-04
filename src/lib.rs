//! PNGer - PNG Payload Embedder & Extracter
//!
//! A library for embedding & extracting payloads within PNG files

use std::io::{BufWriter, Cursor};

// Module declarations
pub mod error;
pub mod io;
pub mod strategy;
pub mod utils;

// Re-exports for public API
pub use error::PngerError;
pub use strategy::{Mode, Strategy};

use io::read_file;
use strategy::get_strategy;
use utils::setup_png_encoder;

/// Decode PNG and extract info
fn decode_png_info(png_data: &[u8]) -> Result<(png::Reader<Cursor<&[u8]>>, png::Info), PngerError> {
    let decoder = png::Decoder::new(Cursor::new(png_data));
    let reader = decoder.read_info()?;
    let info = reader.info().clone();
    Ok((reader, info))
}

/// Read raw image data from PNG reader
fn read_image_data(reader: &mut png::Reader<Cursor<&[u8]>>) -> Result<Vec<u8>, PngerError> {
    let mut image_data = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut image_data)?;
    Ok(image_data)
}

/// Encode PNG with modified image data
fn encode_png_with_data(info: &png::Info, image_data: &[u8]) -> Result<Vec<u8>, PngerError> {
    let mut writer_buffer = BufWriter::new(Vec::new());
    let encoder = setup_png_encoder(info, &mut writer_buffer)?;
    
    let mut writer = encoder.write_header()?;
    writer.write_image_data(image_data)?;
    writer.finish()?;
    
    writer_buffer.into_inner().map_err(|e| PngerError::IoError {
        message: format!("Failed to extract buffer: {}", e)
    })
}

/// Embed payload using specified strategy
fn embed_with_strategy(
    image_data: &mut [u8],
    payload_data: &[u8],
    strategy: Box<dyn Strategy>,
) -> Result<(), PngerError> {
    strategy.embed(image_data, payload_data)
}

/// Embed a payload into PNG data (memory-based API)
///
/// Takes PNG data as a byte array and embeds the payload data into it.
/// This is the core function used internally by the file-based API.
pub fn embed_payload_from_bytes(
    png_data: &[u8],
    payload_data: &[u8],
) -> Result<Vec<u8>, PngerError> {
    embed_payload_from_bytes_with_mode(png_data, payload_data, Mode::default())
}

/// Embed a payload into PNG data with specified mode
pub fn embed_payload_from_bytes_with_mode(
    png_data: &[u8],
    payload_data: &[u8],
    mode: Mode,
) -> Result<Vec<u8>, PngerError> {
    let (mut reader, info) = decode_png_info(png_data)?;
    let mut image_data = read_image_data(&mut reader)?;
    
    let strategy = get_strategy(mode);
    embed_with_strategy(&mut image_data, payload_data, strategy)?;
    
    encode_png_with_data(&info, &image_data)
}

/// Embed a payload into a PNG file (file-based API)
///
/// Takes a file path to a PNG image and embeds the payload data into it.
/// Handles file I/O internally and is the primary interface for most use cases.
pub fn embed_payload_from_file(png_path: &str, payload_data: &[u8]) -> Result<Vec<u8>, PngerError> {
    embed_payload_from_file_with_mode(png_path, payload_data, Mode::default())
}

/// Embed a payload into a PNG file with specified mode
pub fn embed_payload_from_file_with_mode(
    png_path: &str,
    payload_data: &[u8],
    mode: Mode,
) -> Result<Vec<u8>, PngerError> {
    let png_data = read_file(png_path)?;
    embed_payload_from_bytes_with_mode(&png_data, payload_data, mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use strategy::lsb::LSBStrategy;

    #[test]
    fn test_embed_payload_from_bytes() {
        let png_data = b"fake png data";
        let payload = b"test payload";
        // This will fail with fake PNG data, but we're testing the API exists
        let result = embed_payload_from_bytes(png_data, payload);
        assert!(result.is_err()); // Should fail because it's not valid PNG data
    }

    #[test]
    fn test_embed_payload_from_file() {
        // This test would require a real PNG file
        // For now, we'll test the error case
        let payload = b"test payload";
        let result = embed_payload_from_file("nonexistent.png", payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_mode_enum() {
        let mode = Mode::LSB;
        assert_eq!(mode, Mode::default());
    }

    #[test]
    fn test_lsb_strategy() {
        let strategy = LSBStrategy;
        let image_data = vec![0xFF; 1000]; // Fake image data
        let capacity = strategy.max_capacity(&image_data);
        assert_eq!(capacity, (1000 - 32) / 8); // Should reserve 32 bits for length
    }
}
