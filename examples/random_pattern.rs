//! Random Pattern LSB Example
//!
//! LSB steganography with pseudorandom pixel selection for better security.

use pnger::strategy::lsb::LSBConfig;
use pnger::{
    EmbeddingOptions, Strategy, embed_payload_from_file_with_options,
    extract_payload_from_bytes_with_options,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_path = "examples/assets/car.png";
    let message = "Secret message using random LSB pattern with auto-generated seed";

    // Configure random pattern with auto-generated seed
    let strategy = Strategy::LSB(LSBConfig::random());
    let options = EmbeddingOptions::new(strategy);

    // Embed payload
    println!("Embedding: '{message}'");
    let image_with_payload =
        embed_payload_from_file_with_options(image_path, message.as_bytes(), options.clone())?;

    // Extract payload (seed automatically recovered from image)
    let extracted = extract_payload_from_bytes_with_options(&image_with_payload, options)?;
    let extracted_message = String::from_utf8(extracted)?;

    println!("Extracted: '{extracted_message}'");
    println!("Match: {}", message == extracted_message);

    Ok(())
}
