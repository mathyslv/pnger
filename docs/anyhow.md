# Anyhow Documentation

**Library ID**: `/dtolnay/anyhow`  
**Description**: Flexible concrete Error type built on std::error::Error  
**Code Snippets**: 9  
**Trust Score**: 9.3  

## Overview

Anyhow provides a trait object based error handling for Rust applications. It's designed for ease of use and provides a single error type that can represent any error that implements `std::error::Error`.

## Dependencies

Add to your `Cargo.toml`:
```toml
[dependencies]
anyhow = "1.0"
```

For `no_std` environments:
```toml
[dependencies]
anyhow = { version = "1.0", default-features = false }
```

## Basic Usage

### Result Type Alias

Use `anyhow::Result<T>` instead of `Result<T, E>`:

```rust
use anyhow::Result;

fn get_cluster_info() -> Result<ClusterMap> {
    let config = std::fs::read_to_string("cluster.json")?;
    let map: ClusterMap = serde_json::from_str(&config)?;
    Ok(map)
}
```

This is equivalent to `Result<ClusterMap, anyhow::Error>`.

### Error Propagation

The `?` operator works with any error that implements `std::error::Error`:

```rust
use anyhow::Result;

fn read_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.toml")?;  // io::Error
    let config: Config = toml::from_str(&content)?;         // toml::de::Error
    Ok(config)
}
```

## Adding Context

### Using Context Trait

Add descriptive context to errors:

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let path = "important_file.txt";
    let content = std::fs::read(path)
        .with_context(|| format!("Failed to read instrs from {}", path))?;
    
    process_data(&content)
        .context("Failed to process important data")?;
    
    Ok(())
}
```

### Context vs with_context

- `context()`: Takes a static string or string literal
- `with_context()`: Takes a closure that returns the context (evaluated lazily)

```rust
// Static context
.context("Something went wrong")?;

// Dynamic context (evaluated only if error occurs)
.with_context(|| format!("Failed to process file: {}", filename))?;
```

## Creating Custom Errors

### One-off Errors

Use the `anyhow!` macro:

```rust
use anyhow::anyhow;

fn validate_config(config: &Config) -> Result<()> {
    if config.timeout == 0 {
        return Err(anyhow!("Timeout cannot be zero"));
    }
    
    if config.retries > 10 {
        return Err(anyhow!("Too many retries: {}, max is 10", config.retries));
    }
    
    Ok(())
}
```

### Early Return with bail!

Use `bail!` for early error returns:

```rust
use anyhow::{bail, Result};

fn process_request(request: &Request) -> Result<Response> {
    if request.auth_token.is_empty() {
        bail!("Missing auth token");
    }
    
    if request.data.is_empty() {
        bail!("Request data cannot be empty");
    }
    
    // Process request...
    Ok(Response::new())
}
```

## Error Inspection

### Downcasting

Extract specific error types:

```rust
use anyhow::Result;

fn handle_error(error: anyhow::Error) -> Result<String> {
    // Check for specific error types
    if let Some(io_err) = error.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            std::io::ErrorKind::NotFound => {
                return Ok("File not found, using defaults".to_string());
            }
            std::io::ErrorKind::PermissionDenied => {
                return Ok("Permission denied, skipping".to_string());
            }
            _ => {}
        }
    }
    
    if let Some(custom_err) = error.downcast_ref::<MyCustomError>() {
        return Ok(format!("Custom error: {}", custom_err));
    }
    
    Err(error)
}
```

### Error Chain Iteration

Walk through the error chain:

```rust
fn print_error_chain(err: &anyhow::Error) {
    eprintln!("Error: {}", err);
    
    let mut source = err.source();
    while let Some(err) = source {
        eprintln!("Caused by: {}", err);
        source = err.source();
    }
}
```

## Integration with Custom Error Types

Anyhow works seamlessly with `thiserror`:

```rust
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid config format")]
    InvalidFormat,
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

fn load_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.toml")
        .context("Failed to read config file")?;
    
    parse_config(&content)
        .context("Failed to parse configuration")?;
    
    Ok(config)
}

fn parse_config(content: &str) -> Result<Config, ConfigError> {
    // This can return ConfigError, which automatically converts to anyhow::Error
    if content.is_empty() {
        return Err(ConfigError::MissingField { 
            field: "content".to_string() 
        });
    }
    
    // Parse and return config...
    Ok(Config::default())
}
```

## Error Display

Anyhow provides rich error formatting:

```rust
// Error display example
Error: Failed to read instrs from ./path/to/instrs.json

Caused by:
    No such file or directory (os error 2)
```

## Advanced Patterns

### Conditional Error Handling

```rust
use anyhow::{Context, Result};

fn process_file(path: &str, ignore_missing: bool) -> Result<()> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            process_content(&content)?;
        }
        Err(e) if ignore_missing && e.kind() == std::io::ErrorKind::NotFound => {
            println!("File {} not found, skipping", path);
        }
        Err(e) => {
            return Err(e).with_context(|| format!("Failed to read {}", path));
        }
    }
    Ok(())
}
```

### Result Extension Methods

```rust
use anyhow::{Context, Result};

trait ResultExt<T> {
    fn with_path_context(self, path: &str) -> Result<T>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn with_path_context(self, path: &str) -> Result<T> {
        self.with_context(|| format!("Error processing file: {}", path))
    }
}

// Usage
fn process_files(paths: &[String]) -> Result<()> {
    for path in paths {
        let content = std::fs::read_to_string(path)
            .with_path_context(path)?;
        
        process_content(&content)
            .with_path_context(path)?;
    }
    Ok(())
}
```

## For PNGer Project

Here's how you might use anyhow in your PNG steganography tool:

```rust
use anyhow::{Context, Result, bail};
use std::path::Path;

// Main function with anyhow error handling
fn main() -> Result<()> {
    let args = parse_args()?;
    
    let png_data = std::fs::read(&args.png_file)
        .with_context(|| format!("Failed to read PNG file: {}", args.png_file.display()))?;
    
    let payload = if let Some(payload_file) = &args.payload_file {
        std::fs::read(payload_file)
            .with_context(|| format!("Failed to read payload file: {}", payload_file.display()))?
    } else if let Some(payload_text) = &args.payload {
        payload_text.as_bytes().to_vec()
    } else {
        bail!("Either --payload or --payload-file must be specified");
    };
    
    let result = embed_payload_from_bytes(&png_data, &payload)
        .context("Failed to embed payload in PNG")?;
    
    match (&args.output, args.raw) {
        (Some(output_path), false) => {
            std::fs::write(output_path, &result)
                .with_context(|| format!("Failed to write output to: {}", output_path.display()))?;
            println!("Payload embedded successfully to {}", output_path.display());
        }
        (None, true) => {
            use std::io::{self, Write};
            io::stdout().write_all(&result)
                .context("Failed to write to stdout")?;
        }
        (None, false) => {
            bail!("Output method required: use --output <file> or --raw");
        }
        (Some(_), true) => {
            bail!("Cannot use both --output and --raw");
        }
    }
    
    Ok(())
}

// Library functions using anyhow
pub fn embed_payload_from_file(png_path: &str, payload_data: &[u8]) -> Result<Vec<u8>> {
    let png_data = std::fs::read(png_path)
        .with_context(|| format!("Failed to read PNG file: {}", png_path))?;
    
    embed_payload_from_bytes(&png_data, payload_data)
}

pub fn embed_payload_from_bytes(png_data: &[u8], payload_data: &[u8]) -> Result<Vec<u8>> {
    if png_data.len() < 8 {
        bail!("Invalid PNG file: too small");
    }
    
    // Check PNG signature
    if &png_data[0..8] != &[137, 80, 78, 71, 13, 10, 26, 10] {
        bail!("Invalid PNG file: bad signature");
    }
    
    if payload_data.is_empty() {
        bail!("Payload cannot be empty");
    }
    
    // Estimate maximum payload size based on image
    let estimated_pixels = estimate_pixel_count(png_data)?;
    let max_payload_size = estimated_pixels / 8; // 1 bit per pixel
    
    if payload_data.len() > max_payload_size {
        bail!(
            "Payload too large: {} bytes, estimated maximum: {} bytes", 
            payload_data.len(), 
            max_payload_size
        );
    }
    
    // Perform the embedding
    embed_using_lsb(png_data, payload_data)
        .context("LSB embedding failed")
}

fn estimate_pixel_count(png_data: &[u8]) -> Result<usize> {
    // Simple estimation - in real implementation you'd parse PNG headers
    Ok(1000000) // Placeholder
}

fn embed_using_lsb(png_data: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    // Implementation would go here
    Ok(png_data.to_vec())
}
```

## Best Practices

1. **Use `anyhow::Result<T>`** for application code
2. **Use `thiserror`** for library error types
3. **Add context** to all operations that can fail
4. **Use `with_context(|| format!(...))` for dynamic context**
5. **Use `bail!` for early returns with custom messages**
6. **Consider downcasting** for specific error handling
7. **Chain errors** to preserve the full error context

Anyhow provides excellent ergonomics for error handling in applications while maintaining the flexibility to work with any error type.