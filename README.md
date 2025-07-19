# PNGer

![Crates.io MSRV](https://img.shields.io/crates/msrv/pnger)
[![Crates.io](https://img.shields.io/crates/v/pnger?style=flat-square)](https://crates.io/crates/pnger)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)
[![Build Status](https://img.shields.io/github/actions/workflow/status/mathyslv/pnger/ci.yml?branch=main&style=flat-square)](https://github.com/mathyslv/pnger/actions/workflows/ci.yml?query=branch%3Amain)
[![Coverage Status](https://img.shields.io/coveralls/github/mathyslv/pnger/main?style=flat-square)](https://coveralls.io/github/mathyslv/pnger?branch=main)
[![Contributors](https://img.shields.io/github/contributors/mathyslv/pnger?style=flat-square)](https://github.com/mathyslv/pnger/graphs/contributors)


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
git clone https://github.com/user/pnger.git
cd pnger
cargo build --release
```

The binary will be available at `target/release/pnger`.

### Using Cargo
```bash
cargo install pnger
```

## Usage

### Basic Examples

Embed a text file into a PNG image:
```bash
pnger -i photo.png -p secret.txt -o output.png
```

Embed binary data with explicit LSB mode:
```bash
pnger -i image.png -p payload.bin -o steganographic.png --mode lsb
```

Output to stdout for piping:
```bash
pnger -i image.png -p data.json --raw > result.png
```

### Advanced Examples

Advanced LSB with reproducible random pattern:
```bash
pnger -i photo.png -p secret.txt -o output.png --lsb-seed "myseed" --lsb-salt "verylongsalt"
```

Linear LSB pattern for sequential embedding:
```bash
pnger -i image.png -p data.bin -o result.png --lsb-pattern linear
```

Target specific bit position (bit 2 instead of LSB):
```bash
pnger -i photo.png -p payload.json -o steganographic.png --lsb-bit-index 2
```

Add XOR obfuscation for extra security:
```bash
pnger -i image.png -p sensitive.txt -o encrypted.png --xor --xor-key "secretkey"
```

Extract with matching parameters:
```bash
pnger -x -i encrypted.png -o extracted.txt --lsb-seed "myseed" --lsb-salt "verylongsalt" --xor --xor-key "secretkey"
```

### Command Line Options

```
Usage: pnger [OPTIONS] --input <FILE>

Options:
  -i, --input <FILE>              Input PNG file
  -p, --payload <FILE>            Payload file to embed
  -o, --output <FILE>             Output file (write result to file)
      --raw                       Output raw binary data to stdout
  -m, --mode <MODE>               Embedding mode to use [default: lsb]
  -x, --extract                   Extract payload from input file
      --xor                       Toggle payload obfuscation with XOR algorithm
      --xor-key <XOR_KEY>         Key to use for XOR obfuscation
      --lsb-pattern <PATTERN>     LSB pattern to use (linear or random) [default: random]
      --lsb-bit-index <INDEX>     LSB target bit index (0-7) [default: 0]
      --lsb-seed <SEED>           LSB seed for reproducible random patterns
      --lsb-salt <SALT>           LSB salt for reproducible random patterns (minimum 8 characters)
  -h, --help                      Print help
  -V, --version                   Print version

Available modes:
  lsb    Least Significant Bit embedding

Available LSB patterns:
  linear    Sequential bit embedding
  random    Pseudo-random bit embedding (default)
```

## Library Usage

PNGer can also be used as a Rust library:

```rust
use pnger::{EmbeddingOptions, EmbeddingStrategy, RandomOptions, LinearOptions, Obfuscation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Basic embedding with random LSB
    let strategy = EmbeddingStrategy::Random(RandomOptions::default());
    let options = EmbeddingOptions::new(strategy);
    
    // Reproducible random with seed and salt
    let strategy = EmbeddingStrategy::Random(
        RandomOptions::new()
            .with_seed(b"myseed".to_vec())
            .with_salt(b"verylongsalt".to_vec())
    );
    let mut options = EmbeddingOptions::new(strategy);
    
    // Add XOR obfuscation
    options.obfuscation(Obfuscation::Xor { 
        key: b"secretkey".to_vec() 
    });
    
    // Linear pattern with custom bit index
    let strategy = EmbeddingStrategy::Linear(
        LinearOptions::new().with_bit_index(3)
    );
    let options = EmbeddingOptions::new(strategy);
    
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

- Rust 1.70 or higher
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
