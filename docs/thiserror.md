# Thiserror Documentation

**Library ID**: `/dtolnay/thiserror`  
**Description**: derive(Error) for struct and enum error types  
**Code Snippets**: 11  
**Trust Score**: 9.3  

## Overview

Thiserror is a library that provides a convenient derive macro for the standard library's `std::error::Error` trait. It eliminates much of the boilerplate code required for implementing custom error types.

## Dependencies

Add to your `Cargo.toml`:
```toml
[dependencies]
thiserror = "2"
```

## Basic Usage

### Simple Error Enum

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),
    
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    
    #[error("unknown data store error")]
    Unknown,
}
```

### Error Struct

```rust
use thiserror::Error;

#[derive(Error, Debug)]
#[error("oops something went wrong")]
pub struct MyError;
```

## Key Features

### Automatic Display Implementation

The `#[error("...")]` attribute automatically generates a `Display` implementation:

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid rdo_lookahead_frames {0} (expected < {max})", max = i32::MAX)]
    InvalidLookahead(u32),
}
```

### Field Interpolation

Reference fields in error messages:

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("first letter must be lowercase but was {:?}", first_char(.0))]
    WrongCase(String),
    
    #[error("invalid index {idx}, expected at least {} and at most {}", .limits.lo, .limits.hi)]
    OutOfBounds { idx: usize, limits: Limits },
}
```

### Automatic From Implementation

Use `#[from]` to automatically generate `From` trait implementations:

```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    
    #[error("Glob error")]
    Glob(#[from] globset::Error),
}

// This automatically generates:
// impl From<std::io::Error> for MyError { ... }
// impl From<globset::Error> for MyError { ... }
```

### Error Source Chain

Mark fields as error sources with `#[source]`:

```rust
#[derive(Error, Debug)]
pub struct MyError {
    msg: String,
    #[source]  // optional if field name is `source`
    source: anyhow::Error,
}
```

### Transparent Errors

Use `#[error(transparent)]` to forward `source()` and `Display` to underlying error:

```rust
#[derive(Error, Debug)]
pub enum MyError {
    #[error("validation failed")]
    Validation,
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),  // source and Display delegate to anyhow::Error
}
```

### Backtrace Support

Automatic backtrace support (requires nightly Rust 1.73+):

```rust
use std::backtrace::Backtrace;

#[derive(Error, Debug)]
pub struct MyError {
    msg: String,
    backtrace: Backtrace,  // automatically detected
}
```

Forwarding backtraces from source errors:

```rust
#[derive(Error, Debug)]
pub enum MyError {
    Io {
        #[backtrace]
        source: std::io::Error,
    },
}
```

Automatic backtrace capture with `#[from]`:

```rust
#[derive(Error, Debug)]
pub enum MyError {
    Io {
        #[from]
        source: std::io::Error,
        backtrace: Backtrace,
    },
}
```

## Advanced Patterns

### Opaque Error Wrapper

Hide internal error representation:

```rust
// PublicError is public, but opaque and easy to keep compatible.
#[derive(Error, Debug)]
#[error(transparent)]
pub struct PublicError(#[from] ErrorRepr);

impl PublicError {
    // Accessors for anything we do want to expose publicly.
}

// Private and free to change across minor version of the crate.
#[derive(Error, Debug)]
enum ErrorRepr {
    #[error("something went wrong")]
    SomethingWrong,
    // Can add more variants without breaking changes
}
```

### Complex Error Hierarchies

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
    Database(#[from] DatabaseError),
    
    #[error("Network error")]
    Network(#[from] NetworkError),
    
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Validation failed")]
    Validation(#[from] ValidationError),
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed")]
    Connection(#[from] std::io::Error),
    
    #[error("Query failed: {query}")]
    Query { query: String },
}

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("HTTP error: {status}")]
    Http { status: u16 },
    
    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid format for field {field}: {value}")]
    InvalidFormat { field: String, value: String },
}
```

## For PNGer Project

Here's how you might structure errors for your PNG steganography tool:

```rust
use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum PngError {
    #[error("I/O error")]
    Io(#[from] io::Error),
    
    #[error("PNG decoding error")]
    Decode(#[from] png::DecodingError),
    
    #[error("PNG encoding error")]
    Encode(#[from] png::EncodingError),
    
    #[error("Invalid PNG file: {reason}")]
    InvalidPng { reason: String },
    
    #[error("Payload too large: {size} bytes, maximum {max} bytes")]
    PayloadTooLarge { size: usize, max: usize },
    
    #[error("Unsupported PNG format: {details}")]
    UnsupportedFormat { details: String },
    
    #[error("Steganography error: {message}")]
    Steganography { message: String },
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("PNG processing error")]
    Png(#[from] PngError),
    
    #[error("Invalid arguments: {message}")]
    InvalidArgs { message: String },
    
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },
    
    #[error("Output method required: use --output <file> or --raw")]
    OutputMethodRequired,
}

// Usage example
fn embed_payload_from_file(png_path: &str, payload_data: &[u8]) -> Result<Vec<u8>, PngError> {
    let png_data = std::fs::read(png_path)?;
    embed_payload_from_bytes(&png_data, payload_data)
}

fn embed_payload_from_bytes(png_data: &[u8], payload_data: &[u8]) -> Result<Vec<u8>, PngError> {
    if payload_data.len() > 1_000_000 {
        return Err(PngError::PayloadTooLarge {
            size: payload_data.len(),
            max: 1_000_000,
        });
    }
    
    // Implementation here...
    Ok(vec![])
}
```

## Integration with Anyhow

Thiserror works seamlessly with `anyhow`:

```rust
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("custom error occurred")]
    Custom,
}

fn my_function() -> Result<()> {
    // Can return custom errors directly
    Err(MyError::Custom)?
}
```

## Best Practices

1. **Use descriptive error messages** with field interpolation
2. **Chain errors** using `#[from]` and `#[source]` for context
3. **Keep error types focused** - one enum per module/domain
4. **Use transparent errors** for error forwarding
5. **Include relevant context** in error messages
6. **Consider backwards compatibility** when adding new error variants

This provides a robust foundation for error handling in your Rust applications with minimal boilerplate.