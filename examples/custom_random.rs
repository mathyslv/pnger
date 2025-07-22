//! Custom Random Pattern LSB Example
//!
//! Random LSB steganography with password-derived seed and custom bit index.

use pnger::strategy::lsb::{BitIndex, LSBConfig};
use pnger::{
    EmbeddingOptions, Strategy, embed_payload_from_file_with_options,
    extract_payload_from_bytes_with_options,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_path = "examples/assets/car.png";
    let message = "Password-protected message with custom bit index";
    let password = "MySecurePassword123";

    // Configure random pattern with password-derived seed and custom bit index
    let strategy = Strategy::LSB(
        LSBConfig::random()
            .with_password(password.to_string())
            .with_bit_index(BitIndex::Bit1), // Use bit index 1 instead of 0
    );
    let options = EmbeddingOptions::new(strategy);

    // Embed payload
    println!("Embedding with password: '{message}'");
    let image_with_payload =
        embed_payload_from_file_with_options(image_path, message.as_bytes(), options.clone())?;

    // Extract payload (requires same password)
    let extracted = extract_payload_from_bytes_with_options(&image_with_payload, options)?;
    let extracted_message = String::from_utf8(extracted)?;

    println!("Extracted: '{extracted_message}'");
    println!("Match: {}", message == extracted_message);

    // Demonstrate wrong password fails
    let wrong_strategy = Strategy::LSB(
        LSBConfig::random()
            .with_password("WrongPassword".to_string())
            .with_bit_index(BitIndex::Bit1),
    );
    let wrong_options = EmbeddingOptions::new(wrong_strategy);

    if let Ok(wrong_payload) =
        extract_payload_from_bytes_with_options(&image_with_payload, wrong_options)
    {
        let garbled = String::from_utf8_lossy(&wrong_payload);
        println!(
            "Wrong password result: '{}'",
            &garbled[..20.min(garbled.len())]
        );
    }

    Ok(())
}
