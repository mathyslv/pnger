# PNGer

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/user/pnger)

A cross-platform command-line tool for embedding payloads within PNG files using steganography techniques.

## Features

- **LSB Steganography**: Embed data using Least Significant Bit manipulation
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

### Command Line Options

```
Usage: pnger [OPTIONS] --input <FILE> --payload <FILE>

Options:
  -i, --input <FILE>     Input PNG file
  -p, --payload <FILE>   Payload file to embed
  -o, --output <FILE>    Output file (write result to file)
      --raw              Output raw binary data to stdout
  -m, --mode <MODE>      Steganography mode to use [default: lsb]
  -h, --help             Print help
  -V, --version          Print version

Available modes:
  lsb    Least Significant Bit embedding
```

## Library Usage

PNGer can also be used as a Rust library:

```rust
use pnger::{embed_payload_from_file_with_mode, Mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = embed_payload_from_file_with_mode(
        "input.png",
        b"secret data",
        Mode::LSB,
    )?;
    
    std::fs::write("output.png", result)?;
    Ok(())
}
```

### API Functions

- `embed_payload_from_file(png_path, payload_data)` - Embed using default mode
- `embed_payload_from_file_with_mode(png_path, payload_data, mode)` - Embed with specific mode
- `embed_payload_from_bytes(png_data, payload_data)` - Memory-based embedding
- `embed_payload_from_bytes_with_mode(png_data, payload_data, mode)` - Memory-based with mode

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