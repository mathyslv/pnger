# PNG Library Documentation

**Library ID**: `/photopea/upng.js` (JavaScript - closest match for PNG manipulation)  
**Description**: Fast and advanced PNG (APNG) decoder and encoder (lossy / lossless)  
**Code Snippets**: 3  
**Trust Score**: 8.4  

## Overview

For Rust PNG handling, you'll typically want to use the `png` crate from crates.io, which provides comprehensive PNG encoding and decoding capabilities.

## Rust PNG Crate

The standard Rust PNG library is the `png` crate:

```toml
[dependencies]
png = "0.17"
```

## Basic PNG Reading

```rust
use png::Decoder;
use std::fs::File;

fn read_png(path: &str) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let decoder = Decoder::new(file);
    let mut reader = decoder.read_info()?;
    
    // Get image info
    let info = reader.info();
    let width = info.width;
    let height = info.height;
    
    // Allocate buffer
    let mut buf = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut buf)?;
    
    Ok((buf, width, height))
}
```

## Basic PNG Writing

```rust
use png::{BitDepth, ColorType, Encoder};
use std::fs::File;
use std::io::BufWriter;

fn write_png(
    path: &str, 
    data: &[u8], 
    width: u32, 
    height: u32
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);
    
    let mut encoder = Encoder::new(w, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    
    let mut writer = encoder.write_header()?;
    writer.write_image_data(data)?;
    
    Ok(())
}
```

## PNG Chunk Manipulation

For steganography, you'll often work with PNG chunks:

```rust
use png::{Decoder, Encoder};
use std::io::{Cursor, Read};

fn read_png_chunks(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let cursor = Cursor::new(data);
    let decoder = Decoder::new(cursor);
    let mut reader = decoder.read_info()?;
    
    // Access chunk information
    let info = reader.info();
    println!("Width: {}, Height: {}", info.width, info.height);
    println!("Color type: {:?}", info.color_type);
    println!("Bit depth: {:?}", info.bit_depth);
    
    Ok(())
}
```

## Metadata and Text Chunks

PNG files can store metadata in text chunks, useful for steganography:

```rust
use png::{Decoder, text_metadata::*};

fn read_png_metadata(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let decoder = Decoder::new(file);
    let mut reader = decoder.read_info()?;
    
    // Read text chunks
    let info = reader.info();
    for text_chunk in &info.utf8_text {
        println!("Key: {}, Text: {}", text_chunk.keyword, text_chunk.text);
    }
    
    Ok(())
}
```

## Writing PNG with Custom Chunks

```rust
use png::{Encoder, BitDepth, ColorType};
use std::fs::File;
use std::io::BufWriter;

fn write_png_with_metadata(
    path: &str,
    image_data: &[u8],
    width: u32,
    height: u32,
    metadata: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);
    
    let mut encoder = Encoder::new(w, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    
    // Add text chunk
    encoder.add_text_chunk("Comment".to_string(), metadata.to_string())?;
    
    let mut writer = encoder.write_header()?;
    writer.write_image_data(image_data)?;
    
    Ok(())
}
```

## Low-Level Chunk Access

For advanced steganography, you might need direct chunk access:

```rust
use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

fn read_png_signature_and_chunks(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    
    // Read PNG signature
    let mut signature = [0u8; 8];
    file.read_exact(&mut signature)?;
    
    if signature != [137, 80, 78, 71, 13, 10, 26, 10] {
        return Err("Not a valid PNG file".into());
    }
    
    // Read chunks
    loop {
        let mut length_bytes = [0u8; 4];
        if file.read_exact(&mut length_bytes).is_err() {
            break; // End of file
        }
        
        let length = u32::from_be_bytes(length_bytes);
        
        let mut chunk_type = [0u8; 4];
        file.read_exact(&mut chunk_type)?;
        
        println!("Chunk: {}, Length: {}", 
                 String::from_utf8_lossy(&chunk_type), length);
        
        // Skip chunk data and CRC
        file.seek(SeekFrom::Current((length + 4) as i64))?;
        
        if &chunk_type == b"IEND" {
            break;
        }
    }
    
    Ok(())
}
```

## Image Processing

For manipulating pixel data:

```rust
fn modify_png_pixels(
    mut data: Vec<u8>, 
    width: u32, 
    height: u32
) -> Vec<u8> {
    let bytes_per_pixel = 4; // RGBA
    
    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * bytes_per_pixel) as usize;
            
            if index + 3 < data.len() {
                // Modify least significant bit of red channel for steganography
                let payload_bit = 1; // Your payload bit here
                data[index] = (data[index] & 0xFE) | payload_bit;
            }
        }
    }
    
    data
}
```

## Error Handling

```rust
use png::Decoder;
use std::fs::File;

#[derive(Debug)]
enum PngError {
    Io(std::io::Error),
    Png(png::DecodingError),
    InvalidFormat,
}

impl From<std::io::Error> for PngError {
    fn from(err: std::io::Error) -> Self {
        PngError::Io(err)
    }
}

impl From<png::DecodingError> for PngError {
    fn from(err: png::DecodingError) -> Self {
        PngError::Png(err)
    }
}

fn safe_read_png(path: &str) -> Result<(Vec<u8>, u32, u32), PngError> {
    let file = File::open(path)?;
    let decoder = Decoder::new(file);
    let mut reader = decoder.read_info()?;
    
    let info = reader.info();
    let mut buf = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut buf)?;
    
    Ok((buf, info.width, info.height))
}
```

## For PNGer Project

Here's a basic structure for your steganography needs:

```rust
use png::{Decoder, Encoder, BitDepth, ColorType};
use std::fs::File;
use std::io::BufWriter;

pub fn embed_payload_in_png(
    png_path: &str,
    payload: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Read original PNG
    let file = File::open(png_path)?;
    let decoder = Decoder::new(file);
    let mut reader = decoder.read_info()?;
    
    let info = reader.info();
    let mut image_data = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut image_data)?;
    
    // Embed payload using LSB steganography
    let modified_data = embed_lsb(&image_data, payload);
    
    // Create new PNG
    let mut output = Vec::new();
    {
        let mut encoder = Encoder::new(&mut output, info.width, info.height);
        encoder.set_color(info.color_type);
        encoder.set_depth(info.bit_depth);
        
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&modified_data)?;
    }
    
    Ok(output)
}

fn embed_lsb(image_data: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut result = image_data.to_vec();
    // Implement your LSB embedding logic here
    result
}
```