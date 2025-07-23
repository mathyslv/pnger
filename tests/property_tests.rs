//! Property-based tests for PNGer library
//!
//! These tests verify core properties that should always hold true:
//! 1. Roundtrip property: embed → extract always preserves payload
//! 2. Different configurations produce different results
//! 3. Deterministic: same inputs produce same outputs

use pnger::{
    embed_payload_from_bytes_with_options, extract_payload_from_bytes_with_options,
    EmbeddingOptions,
};
use proptest::prelude::*;

// Strategy for generating valid PNG test images
fn png_strategy() -> impl Strategy<Value = Vec<u8>> {
    (100..150u32, 100..150u32, 0..255u8)
        .prop_map(|(width, height, color)| create_simple_png(width, height, [color, color, color]))
}

// Strategy for generating payloads
fn payload_strategy() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..20)
}

fn embedding_options_strategy() -> impl Strategy<Value = EmbeddingOptions> {
    // Generate various option combinations
    prop_oneof![
        Just(EmbeddingOptions::linear()),
        Just(EmbeddingOptions::random()),
        "[a-zA-Z0-9]{2,16}".prop_map(EmbeddingOptions::random_with_password),
    ]
}

/// Create a simple but valid PNG image
fn create_simple_png(width: u32, height: u32, color: [u8; 3]) -> Vec<u8> {
    use std::io::Cursor;

    // Create RGB image data
    let mut image_data = Vec::new();
    for _ in 0..height {
        for _ in 0..width {
            image_data.extend_from_slice(&color);
        }
    }

    // Create PNG using the png crate
    let mut png_data = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png_data);
        let mut encoder = png::Encoder::new(&mut cursor, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().expect("ahbon");
        writer.write_image_data(&image_data).unwrap();
    }

    png_data
}

fn can_embed_payload<P1: AsRef<[u8]>, P2: AsRef<[u8]>>(png_data: P1, payload: P2) -> bool {
    (payload.as_ref().len() * 8 + 72) < png_data.as_ref().len()
}

proptest! {
    /// Core property: embed then extract should preserve payload data
    #[cfg_attr(tarpaulin, ignore)]
    #[test]
    fn roundtrip_preserves_payload(
          png_data in png_strategy(),
          payload in payload_strategy(),
          options in embedding_options_strategy()
      ) {
          // Skip if payload too large for image
          prop_assume!(can_embed_payload(&png_data, &payload));

          // Embed payload
          let embedded = embed_payload_from_bytes_with_options(&png_data, &payload, options.clone())
              .map_err(|e| TestCaseError::Fail(format!("Embed failed: {e}").into()))?;

          // Extract payload
          let extracted = extract_payload_from_bytes_with_options(&embedded, options)
              .map_err(|e| TestCaseError::Fail(format!("Extract failed: {e}").into()))?;

          // Verify they match
          prop_assert_eq!(payload, extracted);
      }

    /// Property: linear embedding is deterministic
    #[cfg_attr(tarpaulin, ignore)]
    #[test]
    fn linear_embedding_is_deterministic(
        payload in prop::collection::vec(any::<u8>(), 1..10)
    ) {
        let png_data = create_simple_png(24, 24, [100, 150, 200]);
        let options = EmbeddingOptions::linear();

        // Embed the same payload twice
        let embedded1 = embed_payload_from_bytes_with_options(&png_data, &payload, options.clone())?;
        let embedded2 = embed_payload_from_bytes_with_options(&png_data, &payload, options)?;

        // Results should be identical for deterministic strategy
        prop_assert_eq!(embedded1, embedded2, "Linear embedding should be deterministic");
    }

    /// Property: XOR obfuscation changes the result
    #[cfg_attr(tarpaulin, ignore)]
    #[test]
    fn xor_obfuscation_changes_result(
        payload in prop::collection::vec(any::<u8>(), 1..4)
    ) {
        let png_data = create_simple_png(24, 24, [75, 125, 175]);
        let options_plain = EmbeddingOptions::linear();
        let options_xor = EmbeddingOptions::linear().with_xor_string("key");

        let embedded_plain = embed_payload_from_bytes_with_options(&png_data, &payload, options_plain)?;
        let embedded_xor = embed_payload_from_bytes_with_options(&png_data, &payload, options_xor)?;

        // XOR obfuscation should produce different results
        prop_assert_ne!(embedded_plain, embedded_xor, "XOR obfuscation should change the result");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_create_simple_png() {
        let png_data = create_simple_png(10, 10, [255, 0, 0]);

        // Should have PNG signature
        assert_eq!(
            &png_data[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );

        // Should be a reasonable size for a 10x10 image
        assert!(png_data.len() > 100);
        assert!(png_data.len() < 1000);
    }

    #[test]
    fn test_basic_roundtrip() {
        // Simple unit test version of the property test
        let png_data = create_simple_png(16, 16, [128, 128, 128]);
        let payload = b"test";
        let options = EmbeddingOptions::linear();

        let embedded =
            embed_payload_from_bytes_with_options(&png_data, payload, options.clone()).unwrap();
        let extracted = extract_payload_from_bytes_with_options(&embedded, options).unwrap();

        assert_eq!(payload.as_slice(), extracted.as_slice());
    }
}
