# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PNGer is a cross-platform tool for embedding & extracting payloads within PNG files using steganography techniques.

### Project Goals

#### Library Architecture
The core library (`lib.rs`) provides two main entry points for PNG payload embedding and extracting:

1. **File-based API**: `embed_payload_from_file(png_path: &str, payload_data: &[u8]) -> Result<Vec<u8>, Error>`
   - Takes a file path to a PNG image
   - Handles file I/O internally
   - Primary interface for most use cases

2. **Memory-based API**: `embed_payload_from_bytes(png_data: &[u8], payload_data: &[u8]) -> Result<Vec<u8>, Error>`
   - Takes PNG data as a byte array
   - Used internally by the file-based API
   - Enables in-memory processing for advanced scenarios

3. **File-based API**: `extract_payload_from_file(png_path: &str, payload_data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Error>`
   - Takes a file path to a PNG image
   - Handles file I/O internally
   - Primary interface for most use cases

4. **Memory-based API**: `extract_payload_from_bytes(png_data: &[u8], payload_data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), Error>`
   - Takes PNG data as a byte array
   - Used internally by the file-based API
   - Enables in-memory processing for advanced scenarios

#### Binary Interface
The CLI binary (`main.rs`) uses clap for argument parsing and provides the following interface:

**Required Arguments:**
- Input PNG file path (positional or via flag)
- Payload data (file path or direct input)

**Output Options (mutually exclusive):**
- `-o, --output <FILE>`: Write result to specified file
- `--raw`: Output raw binary data to terminal/stdout
- **Default behavior**: If neither `-o` nor `--raw` is specified, display an error message requiring the user to choose an output method (prevents accidental binary output to terminal)

**Design Rationale:**
- File-based API simplifies common usage patterns
- Memory-based API enables flexible integration and testing
- Explicit output specification prevents terminal corruption from binary data
- Error on missing output choice guides users toward intentional behavior

## Development Commands

### Build

```bash
cargo build
```

### Run

```bash
cargo run -- --help
```

### Test

```bash
cargo test
```

### Check/Lint

```bash
cargo check
cargo clippy
```

## Architecture

The codebase follows a modular structure:

- **main.rs**: Entry point with CLI argument parsing and main logic
- **lib.rs**: Core library

### Key Components

## Git Commit Guidelines

**ALWAYS follow Conventional Commits specification for all commit messages.**

Based on https://www.conventionalcommits.org, commit messages must follow this format:

```
<type>[optional scope]: <description>

[optional body]
```

**NEVER ADD Claude co-author to commit messages**

### Required Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (formatting, etc.)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to the build process or auxiliary tools

### Optional Scopes

- **cli**: Command-line interface changes
- **platform**: Platform-specific code changes
- **deps**: Dependency updates

### Examples

```bash
feat(cli): add JSON output format support
fix(mbr): correct partition table offset calculation
docs: update README with installation instructions
chore(deps): update clap to version 4.0
```

### Breaking Changes

For breaking changes, add `!` after type/scope and include `BREAKING CHANGE:` in footer:

```bash
feat(cli)!: remove deprecated --legacy flag

BREAKING CHANGE: The --legacy flag has been removed. Use --format=legacy instead.
```

### Git Commit Message Guidelines

- The git commit message should always have a title
- Optionally include 3-4 lines maximum in description

## Tool Usage Guidelines

### Code Operations

**USE AS OFTEN AS POSSIBLE the Serena MCP server tools for code-related manipulation** including:
- Parsing code structure
- Adding code
- Modifying code
- Deleting code

### Git Operations

**ALWAYS use the GitHub MCP server tools for all git-related operations** including:
- Reading git status, diffs, logs
- Creating commits and branches
- Managing pull requests and issues
- Any other git operations

### File System Operations

**ALWAYS use the filesystem MCP server tools for all file system operations** including:
- Reading files and directories
- Writing and editing files
- Creating directories
- Moving/renaming files
- Searching for files
- Getting file information

## Dependencies

- `clap`: Command-line argument parsing with derive feature
- `png`: PNG encoding/decoding library
- `thiserror`: Custom error type derivation
- `anyhow`: Flexible error handling

## Documentation References

Comprehensive documentation for the project's dependencies is available in the `docs/` directory:

### Library Documentation Files

- **`docs/clap.md`**: Complete guide to the Clap argument parsing library
  - Derive API usage (recommended approach)
  - Builder API patterns
  - Common argument types (flags, options, positional args, subcommands)
  - Error handling and validation
  - PNGer-specific CLI examples

- **`docs/png.md`**: PNG handling with the Rust `png` crate
  - Basic PNG reading and writing
  - Chunk manipulation for steganography
  - Metadata and text chunk operations
  - Low-level chunk access patterns
  - Error handling strategies
  - Steganography-specific examples

- **`docs/thiserror.md`**: Custom error types with Thiserror
  - Error enum and struct definitions
  - Automatic Display implementations
  - Error chaining with `#[from]` and `#[source]`
  - Transparent error forwarding
  - Backtrace support
  - PNGer error hierarchy examples

- **`docs/anyhow.md`**: Application error handling with Anyhow
  - `anyhow::Result<T>` usage patterns
  - Adding context to errors
  - Error propagation with `?` operator
  - Custom error creation with `anyhow!` and `bail!`
  - Error inspection and downcasting
  - Integration with thiserror
  - PNGer application error handling

### Accessing Documentation

When working on PNGer features:

1. **For CLI argument parsing**: Reference `docs/clap.md` for patterns and examples
2. **For PNG file operations**: Check `docs/png.md` for encoding/decoding techniques
3. **For library error types**: Use `docs/thiserror.md` for structured error definitions
4. **For application error handling**: Consult `docs/anyhow.md` for ergonomic error management

These documents provide practical examples tailored to the PNGer use case and can be referenced throughout development to ensure consistent patterns and best practices.
