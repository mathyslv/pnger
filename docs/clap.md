# Clap Documentation

**Library ID**: `/clap-rs/clap`  
**Description**: A full featured, fast Command Line Argument Parser for Rust  
**Code Snippets**: 213  
**Trust Score**: 7.1  

## Overview

Clap is a powerful command-line argument parser for Rust that makes it easy to build CLI applications with comprehensive argument handling.

## Key Features

- **Derive Macros**: Use `#[derive(Parser)]` for easy setup
- **Builder Pattern**: Programmatic command building
- **Subcommands**: Support for nested commands
- **Validation**: Built-in argument validation
- **Help Generation**: Automatic help text generation
- **Shell Completions**: Generate completion scripts

## Basic Usage Examples

### Derive API (Recommended)
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "myapp")]
#[command(about = "A simple CLI")]
struct Cli {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,
    
    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Adds files to myapp
    Add {
        /// Stuff to add
        #[arg(value_name = "PATH")]
        paths: Vec<String>,
    },
}
```

### Builder API
```rust
use clap::{Arg, ArgAction, Command};

let matches = Command::new("myapp")
    .about("A simple CLI")
    .arg(
        Arg::new("name")
            .short('n')
            .long("name")
            .help("Name of the person to greet")
            .required(true)
    )
    .arg(
        Arg::new("count")
            .short('c')
            .long("count")
            .help("Number of times to greet")
            .default_value("1")
    )
    .get_matches();
```

## Common Patterns

### Required Arguments
```rust
#[derive(Parser)]
struct Args {
    /// Required name argument
    name: String,
    
    /// Optional value with default
    #[arg(short, long, default_value = "default")]
    value: String,
}
```

### Multiple Values
```rust
#[derive(Parser)]
struct Args {
    /// Multiple files
    #[arg(short, long)]
    files: Vec<String>,
}
```

### Flags (Boolean)
```rust
#[derive(Parser)]
struct Args {
    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,
    
    /// Debug level (can be used multiple times)
    #[arg(short, long, action = ArgAction::Count)]
    debug: u8,
}
```

### Enums
```rust
#[derive(Parser)]
struct Args {
    #[arg(value_enum)]
    mode: Mode,
}

#[derive(clap::ValueEnum, Clone)]
enum Mode {
    Fast,
    Slow,
}
```

### Subcommands
```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        #[arg(short, long)]
        name: String,
    },
    Remove {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        force: bool,
    },
}
```

## Error Handling

```rust
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(value_parser = clap::value_parser!(u16).range(1..=65535))]
    port: u16,
}

fn main() {
    let args = Args::parse();
    println!("Port: {}", args.port);
}
```

## Validation

```rust
#[derive(Parser)]
struct Args {
    /// Port number (1-65535)
    #[arg(value_parser = port_in_range)]
    port: u16,
}

fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s.parse()
        .map_err(|_| format!("`{s}` isn't a port number"))?;
    if port > 0 && port <= 65535 {
        Ok(port as u16)
    } else {
        Err(format!("port not in range 1-65535"))
    }
}
```

## Dependencies

Add to your `Cargo.toml`:
```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
```

## Common Use Cases in PNGer

For your PNG steganography tool:

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pnger")]
#[command(about = "A tool for embedding payloads in PNG files")]
struct Cli {
    /// Input PNG file
    #[arg(value_name = "PNG_FILE")]
    png_file: PathBuf,
    
    /// Payload to embed
    #[arg(short, long, value_name = "PAYLOAD")]
    payload: Option<String>,
    
    /// Payload file to embed
    #[arg(short = 'f', long = "payload-file", value_name = "FILE")]
    payload_file: Option<PathBuf>,
    
    /// Output file (use --raw for stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    
    /// Output raw binary to stdout
    #[arg(long, conflicts_with = "output")]
    raw: bool,
}
```

This provides a clean, type-safe interface for your command-line arguments with automatic help generation and validation.