# PNGer

![Crates.io MSRV](https://img.shields.io/crates/msrv/pnger)
[![Crates.io](https://img.shields.io/crates/v/pnger?style=flat-square)](https://crates.io/crates/pnger)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/mathyslv/pnger/ci.yml?branch=main&style=flat-square)](https://github.com/mathyslv/pnger/actions/workflows/ci.yml?query=branch%3Amain)

A cross-platform command-line tool for embedding payloads within PNG files using steganography techniques.

## Features

- **LSB Steganography**: Embed data using Least Significant Bit manipulation
- **LSB Customization**: Linear or random patterns with configurable bit targeting (0-7)
- **Reproducible Patterns**: Use seeds and salts for deterministic embedding/extraction
- **XOR Obfuscation**: Optional payload encryption with custom or default keys
- **Bidirectional Operations**: Both embed and extract capabilities with parameter matching
- **Multiple Output Formats**: Save to file or output raw binary data
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Fast & Efficient**: Optimized Rust implementation
- **Extensible Architecture**: Support for multiple steganography algorithms

## Installation

### From Source
```bash
git clone https://github.com/mathyslv/pnger.git
cd pnger
cargo build --release --features bin
```

The binary will be available at `target/release/pnger`.

### Using Cargo
```bash
cargo install pnger --features bin
```

## Usage

### Basic Examples

Embed payload.json into image.png and save to output.png:
```bash
pnger -i image.png -p payload.json -o output.png
```

Use explicit LSB strategy (default, so optional):
```bash
pnger -i image.png -p payload.bin -o output.png --strategy lsb
```

Output raw binary data to stdout:
```bash
pnger -i image.png -p payload.txt --raw > output.png
```

### Advanced Examples

LSB with password-based pattern (secure, nothing embedded):
```bash
pnger -i image.png -p secret.txt -o output.png --lsb-password "mypassword123"
```

LSB with manual hex seed (32 bytes = 64 hex chars):
```bash
pnger -i image.png -p data.bin -o output.png --lsb-seed "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
```

LSB linear pattern instead of random:
```bash
pnger -i image.png -p data.txt -o output.png --lsb-pattern linear
```

LSB with custom bit index (target bit 3 instead of 0):
```bash
pnger -i image.png -p secret.bin -o output.png --lsb-bit-index 3
```

XOR obfuscation with default key:
```bash
pnger -i image.png -p sensitive.txt -o output.png --xor
```

XOR obfuscation with custom key:
```bash
pnger -i image.png -p data.json -o output.png --xor --xor-key "mykey123"
```

Combined: LSB password + XOR:
```bash
pnger -i image.png -p payload.bin -o output.png --lsb-password "mypassword" --xor --xor-key "encrypt"
```

Extract payload from image.png and save to payload.json:
```bash
pnger -x -i output.png -o payload.json
```

Extract with matching LSB and XOR parameters:
```bash
pnger -x -i output.png -o extracted.txt --lsb-password "mypassword" --xor --xor-key "encrypt"
```

Extract payload to stdout:
```bash
pnger -x -i output.png --raw
```

### Command Line Options

```
Usage: pnger [OPTIONS] --input <FILE>

Options:
  -i, --input <FILE>                   Input PNG file
  -p, --payload <FILE>                 Payload file to embed
  -o, --output <FILE>                  Output file (write result to file)
      --raw                            Output raw result data to stdout
  -s, --strategy <STRATEGY>            Embedding strategy to use [default: lsb]
  -x, --extract                        Extract payload from input file
      --xor                            Toggle payload obfuscation with XOR algorithm
      --xor-key <XOR_KEY>              Key to use for XOR obfuscation
      --lsb-pattern <LSB_PATTERN>      LSB pattern to use (linear or random) [default: random]
      --lsb-bit-index <LSB_BIT_INDEX>  LSB target bit index (0-7) [default: 0]
      --lsb-password <LSB_PASSWORD>    Password for reproducible random patterns (nothing embedded in PNG)
      --lsb-seed <LSB_SEED>            LSB seed for reproducible random patterns (raw 32-byte hex seed)
  -h, --help                           Print help
  -V, --version                        Print version

Available strategies:
  lsb    Least Significant Bit embedding

Available LSB patterns:
  linear    Linear pattern (sequential)
  random    Random pattern (pseudo-random) [default: random]
```

## Library Usage

PNGer can also be used as a Rust library:

```rust
use pnger::{EmbeddingOptions, Strategy, Obfuscation};
use pnger::strategy::lsb::{LSBConfig, BitIndex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Basic embedding with random LSB (default)
    let strategy = Strategy::LSB(LSBConfig::random());
    let options = EmbeddingOptions::new(strategy);
    
    // Password-protected random pattern
    let strategy = Strategy::LSB(
        LSBConfig::random().with_password("my_secret_password".to_string())
    );
    let mut options = EmbeddingOptions::new(strategy);
    
    // Add XOR obfuscation
    options.set_obfuscation(Some(Obfuscation::Xor { 
        key: b"secretkey".to_vec() 
    }));
    
    // Linear pattern with custom bit index
    let strategy = Strategy::LSB(
        LSBConfig::linear().with_bit_index(BitIndex::Bit3)
    );
    let options = EmbeddingOptions::new(strategy);
    
    // Using fluent builder API (recommended)
    let options = EmbeddingOptions::linear()
        .with_xor_string("encryption_key");
    
    let options = EmbeddingOptions::random_with_password("secure_password")
        .with_bit_index(BitIndex::Bit1)
        .with_xor_key(b"additional_layer".to_vec());
    
    Ok(())
}
```

### API Functions

- `embed_payload_from_file(png_path, payload_data)` - Embed using default options
- `embed_payload_from_file_with_options(png_path, payload_data, options)` - Embed with custom options
- `embed_payload_from_bytes(png_data, payload_data)` - Memory-based embedding
- `embed_payload_from_bytes_with_options(png_data, payload_data, options)` - Memory-based with options
- `extract_payload_from_file(png_path)` - Extract using default options
- `extract_payload_from_file_with_options(png_path, options)` - Extract with matching options

## How It Works

PNGer uses steganography to hide data within PNG images by modifying the least significant bits of pixel data. The embedded data includes a length prefix, allowing for reliable extraction while maintaining image quality.

## Requirements

- Rust 1.85.1 or higher
- Valid PNG input files

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Security Notice

This tool is intended for educational purposes and legitimate use cases. Users are responsible for complying with applicable laws and regulations regarding data hiding and steganography.
