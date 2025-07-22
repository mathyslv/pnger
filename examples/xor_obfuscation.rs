//! XOR Obfuscation LSB Example
//!
//! LSB steganography with XOR encryption for dual-layer security.

use pnger::strategy::lsb::LSBConfig;
use pnger::{
    EmbeddingOptions, Obfuscation, Strategy, embed_payload_from_file_with_options,
    extract_payload_from_bytes_with_options,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_path = "examples/assets/car.png";
    let message = "TOP SECRET: Mission code Alpha-7. Coordinates 51.5074, -0.1278 at 23:00.";
    let encryption_key = b"SecretKey123!";

    // Configure random pattern + XOR obfuscation
    let strategy = Strategy::LSB(LSBConfig::random());
    let obfuscation = Obfuscation::Xor {
        key: encryption_key.to_vec(),
    };
    let options = EmbeddingOptions::new_with_obfuscation(strategy, obfuscation);

    // Embed obfuscated payload
    println!("Embedding encrypted: '{message}'");
    let image_with_payload =
        embed_payload_from_file_with_options(image_path, message.as_bytes(), options.clone())?;

    // Extract and decrypt payload
    let extracted = extract_payload_from_bytes_with_options(&image_with_payload, options)?;
    let extracted_message = String::from_utf8(extracted)?;

    println!("Extracted decrypted: '{extracted_message}'");
    println!("Match: {}", message == extracted_message);

    // Demonstrate wrong key fails
    let wrong_obfuscation = Obfuscation::Xor {
        key: b"WrongKey".to_vec(),
    };
    let wrong_options = EmbeddingOptions::new_with_obfuscation(
        Strategy::LSB(LSBConfig::random()),
        wrong_obfuscation,
    );

    if let Ok(wrong_payload) =
        extract_payload_from_bytes_with_options(&image_with_payload, wrong_options)
    {
        let garbled = String::from_utf8_lossy(&wrong_payload);
        println!("Wrong key result: '{}'", &garbled[..30.min(garbled.len())]);
    }

    Ok(())
}
