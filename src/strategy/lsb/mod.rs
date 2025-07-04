//! LSB (Least Significant Bit) steganography implementation.
//!
//! This module provides LSB steganography functionality for embedding and extracting
//! payloads in PNG images. LSB steganography works by modifying the least significant
//! bits of image pixel data to encode payload information.
//!
//! # Key Features
//!
//! - **Multiple embedding patterns**: Linear and random patterns for different security needs
//! - **Flexible bit selection**: Choose which bit position to modify (0-7)
//! - **Cryptographic security**: Password-derived seeds using Argon2 key derivation
//! - **Builder pattern API**: Fluent configuration interface
//! - **Zero-copy extraction**: Efficient payload recovery
//!
//! # Security Considerations
//!
//! - **Linear patterns** are faster but create detectable statistical patterns
//! - **Random patterns** provide better security by distributing changes pseudorandomly
//! - **Password-derived seeds** offer security without storing sensitive data in the image
//! - **Auto-generated seeds** provide maximum entropy but require storage in the image header

#[doc(hidden)]
pub mod crypto;
mod data;
mod header;
#[doc(hidden)]
pub mod utils;

use crate::{error::PngerError, strategy::lsb::data::BodyEmbedder};

/// Configuration for LSB (Least Significant Bit) steganography strategy.
///
/// LSB steganography works by modifying the least significant bits of image pixels
/// to encode payload data. This implementation supports both linear and random
/// embedding patterns for enhanced security.
///
/// # Examples
///
/// ## Basic Linear Embedding
/// ```rust
/// use pnger::strategy::lsb::LSBConfig;
///
/// let config = LSBConfig::linear();
/// ```
///
/// ## Random Pattern with Password
/// ```rust
/// use pnger::strategy::lsb::LSBConfig;
///
/// let config = LSBConfig::random()
///     .with_password("secret_password".to_string())
///     .with_bit_index(2);
/// ```
///
/// ## Custom Bit Position
/// ```rust
/// use pnger::strategy::lsb::LSBConfig;
///
/// // Use bit index 1 instead of 0 for potentially better imperceptibility
/// let config = LSBConfig::linear().with_bit_index(1);
/// ```
///
/// # Performance vs Security Trade-offs
///
/// | Pattern | Speed | Security | Detectability |
/// |---------|-------|----------|---------------|
/// | Linear  | Fast  | Low      | High          |
/// | Random  | Slower| High     | Low           |
///
/// Choose linear patterns for speed, random patterns for security.
#[derive(Debug, Clone)]
pub struct LSBConfig {
    bit_index: u8,
    pattern: EmbeddingPattern,
}

/// Embedding pattern configuration for LSB steganography.
///
/// Determines how payload bits are distributed across the image pixels.
/// The choice between linear and random patterns significantly affects both
/// performance and security characteristics.
///
/// # Pattern Types
///
/// ## Linear Pattern
/// Embeds payload bits sequentially across image pixels from top-left to bottom-right.
///
/// **Advantages:**
/// - Faster embedding and extraction
/// - Simpler implementation
/// - Deterministic bit ordering
///
/// **Disadvantages:**
/// - Creates detectable statistical patterns
/// - Vulnerable to visual inspection
/// - Easier to detect with analysis tools
///
/// ## Random Pattern
/// Uses a pseudorandom sequence to determine pixel embedding order.
///
/// **Advantages:**
/// - Better security through randomization
/// - Harder to detect visually
/// - Resistant to statistical analysis
///
/// **Disadvantages:**
/// - Slightly slower performance
/// - Requires seed management
/// - More complex implementation
///
/// # Examples
///
/// ```rust
/// use pnger::strategy::lsb::{LSBConfig, EmbeddingPattern, RandomConfig, SeedSource};
///
/// // Linear pattern (fast but less secure)
/// let linear_config = LSBConfig::linear();
///
/// // Random pattern with auto seed (secure, seed stored in image)
/// let random_auto = LSBConfig::random();
///
/// // Random pattern with password (secure, no seed storage)
/// let random_password = LSBConfig::random()
///     .with_password("my_secret".to_string());
/// ```
#[derive(Debug, Clone)]
pub enum EmbeddingPattern {
    /// Sequential embedding from top-left to bottom-right.
    ///
    /// Provides fast performance but creates detectable patterns.
    /// Best used when performance is critical and security is less important.
    Linear,

    /// Pseudorandom embedding order based on a cryptographic seed.
    ///
    /// Provides better security by distributing payload bits randomly
    /// across the image. The randomness is deterministic based on the seed,
    /// ensuring reliable extraction.
    Random(RandomConfig),
}

/// Configuration for random embedding patterns.
///
/// Controls how the pseudorandom sequence is generated for determining
/// pixel embedding order. The seed source affects both security and
/// storage requirements.
///
/// # Seed Management
///
/// The seed is critical for extraction - without the correct seed,
/// the payload cannot be recovered. Choose the seed source based on
/// your security and convenience requirements.
#[derive(Debug, Clone)]
pub struct RandomConfig {
    seed_source: SeedSource,
}

/// Source for generating pseudorandom embedding seeds.
///
/// Determines how the cryptographic seed is generated and managed.
/// Each option provides different trade-offs between security,
/// convenience, and storage requirements.
///
/// # Security Implications
///
/// | Seed Source | Security | Convenience | Storage Needed |
/// |-------------|----------|-------------|----------------|
/// | Auto        | High     | High        | Yes (in image) |
/// | Password    | High     | Medium      | No             |
/// | Manual      | Variable | Low         | No             |
///
/// # Examples
///
/// ```rust
/// use pnger::strategy::lsb::{LSBConfig, SeedSource};
///
/// // Auto-generated seed (easiest to use)
/// let auto_config = LSBConfig::random(); // Uses SeedSource::Auto by default
///
/// // Password-derived seed (good security, no storage needed)
/// let pwd_config = LSBConfig::random()
///     .with_password("my_secure_password".to_string());
///
/// // Manual seed (full control)
/// let manual_seed = [0u8; 32]; // Your 32-byte seed
/// let manual_config = LSBConfig::random()
///     .with_seed(manual_seed);
/// ```
#[derive(Debug, Clone)]
pub enum SeedSource {
    /// Auto-generate cryptographically secure random seed.
    ///
    /// **How it works:**
    /// - Generates a new 32-byte seed using system randomness
    /// - Embeds the seed in the PNG image header
    /// - No additional credentials needed for extraction
    ///
    /// **Pros:**
    /// - Maximum security through true randomness
    /// - No password to remember or manage
    /// - Automatic seed handling
    ///
    /// **Cons:**
    /// - Seed is stored in the image (adds ~32 bytes)
    /// - Anyone with the image can extract if no additional security
    ///
    /// **Best for:** Simple use cases where convenience is important
    /// and the image itself provides sufficient access control.
    Auto,

    /// Derive seed from password using Argon2 key derivation.
    ///
    /// **How it works:**
    /// - Uses Argon2id to derive a 32-byte seed from the password
    /// - No seed data is stored in the image
    /// - Same password must be provided for extraction
    ///
    /// **Pros:**
    /// - No sensitive data stored in the image
    /// - Password-based authentication
    /// - Resistant to brute force attacks (Argon2)
    ///
    /// **Cons:**
    /// - Password must be securely shared/remembered
    /// - Slightly slower due to key derivation
    ///
    /// **Best for:** Scenarios requiring password protection
    /// and where seed storage is undesirable.
    Password(String),

    /// User-provided 32-byte seed for advanced use cases.
    ///
    /// **How it works:**
    /// - Uses the exact seed bytes provided by the user
    /// - No seed data is stored in the image
    /// - Same seed must be provided for extraction
    ///
    /// **Pros:**
    /// - Full control over randomness source
    /// - No key derivation overhead
    /// - Deterministic for testing
    ///
    /// **Cons:**
    /// - User responsible for secure seed generation
    /// - Seed must be securely stored/transmitted
    /// - Easy to use weak seeds
    ///
    /// **Best for:** Advanced users, testing, integration with
    /// existing key management systems.
    Manual([u8; 32]),
}

// Builder pattern implementations for LSBConfig
impl LSBConfig {
    /// Create a new LSB configuration with linear embedding pattern.
    ///
    /// Linear pattern embeds payload bits sequentially across image pixels
    /// from top-left to bottom-right. This provides the fastest performance
    /// but creates detectable statistical patterns.
    ///
    /// # Default Settings
    /// - Bit index: 0 (least significant bit)
    /// - Pattern: Linear (sequential)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// // Basic linear configuration
    /// let config = LSBConfig::linear();
    ///
    /// // Linear with custom bit position
    /// let config = LSBConfig::linear().with_bit_index(1);
    /// ```
    ///
    /// # Performance Characteristics
    /// - **Embedding speed**: Fastest
    /// - **Extraction speed**: Fastest
    /// - **Memory usage**: Minimal
    /// - **Security**: Lowest (easily detectable)
    pub fn linear() -> Self {
        Self {
            bit_index: 0,
            pattern: EmbeddingPattern::Linear,
        }
    }

    /// Create a new LSB configuration with random embedding pattern.
    ///
    /// Random pattern uses a pseudorandom sequence to determine pixel
    /// embedding order, providing better security at the cost of some
    /// performance. By default, uses an auto-generated seed.
    ///
    /// # Default Settings
    /// - Bit index: 0 (least significant bit)
    /// - Pattern: Random with auto-generated seed
    /// - Seed storage: Embedded in image header
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// // Random with auto-generated seed
    /// let config = LSBConfig::random();
    ///
    /// // Random with password-derived seed
    /// let config = LSBConfig::random()
    ///     .with_password("secret123".to_string());
    ///
    /// // Random with manual seed
    /// let seed = [0u8; 32]; // Your 32-byte seed
    /// let config = LSBConfig::random().with_seed(seed);
    /// ```
    ///
    /// # Security Benefits
    /// - Resistant to visual detection
    /// - Harder to analyze statistically
    /// - Distributes changes across entire image
    pub fn random() -> Self {
        Self {
            bit_index: 0,
            pattern: EmbeddingPattern::Random(RandomConfig {
                seed_source: SeedSource::Auto,
            }),
        }
    }

    /// Set the bit index position for embedding (0-7).
    ///
    /// Determines which bit position in each color channel will be modified.
    /// Lower indices (0-2) provide better capacity but are more detectable.
    /// Higher indices (3-7) are more detectable but may corrupt the image.
    ///
    /// # Parameters
    /// - `index`: Bit position (0 = least significant, 7 = most significant)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// // Use the 2nd least significant bit
    /// let config = LSBConfig::linear().with_bit_index(1);
    ///
    /// // Use the 3rd least significant bit for more security
    /// let config = LSBConfig::random().with_bit_index(2);
    /// ```
    ///
    /// # Bit Index Recommendations
    /// | Index | Visual Impact | Detectability | Recommended Use |
    /// |-------|---------------|---------------|------------------|
    /// | 0     | Minimal       | Low           | General purpose |
    /// | 1     | Minimal       | Medium        | Better security |
    /// | 2     | Low           | Medium        | High security   |
    /// | 3+    | Noticeable    | High          | Not recommended |
    pub fn with_bit_index(mut self, index: u8) -> Self {
        self.bit_index = index;
        self
    }

    /// Set password for random pattern seed derivation.
    ///
    /// Configures the random pattern to derive its seed from the provided
    /// password using Argon2id key derivation. This provides strong security
    /// without storing sensitive data in the image.
    ///
    /// **Note:** Only works with random patterns. Calling this on a linear
    /// configuration has no effect.
    ///
    /// # Parameters
    /// - `password`: Password string for seed derivation
    ///
    /// # Security Features
    /// - Uses Argon2id for password-based key derivation
    /// - Resistant to rainbow table attacks
    /// - Configurable work factors for brute force resistance
    /// - No password or seed data stored in the image
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// let config = LSBConfig::random()
    ///     .with_password("my_secure_password".to_string());
    ///
    /// // Can be chained with other options
    /// let config = LSBConfig::random()
    ///     .with_password("secret".to_string())
    ///     .with_bit_index(1);
    /// ```
    ///
    /// # Password Guidelines
    /// - Use strong, unique passwords
    /// - Minimum 12 characters recommended
    /// - Include mix of letters, numbers, symbols
    /// - Store passwords securely
    pub fn with_password(mut self, password: String) -> Self {
        if let EmbeddingPattern::Random(ref mut config) = self.pattern {
            config.seed_source = SeedSource::Password(password);
        }
        self
    }

    /// Set manual 32-byte seed for random pattern.
    ///
    /// Provides direct control over the pseudorandom seed used for
    /// embedding pattern generation. This is for advanced use cases
    /// where you need deterministic behavior or integration with
    /// existing key management systems.
    ///
    /// **Note:** Only works with random patterns. Calling this on a linear
    /// configuration has no effect.
    ///
    /// # Parameters
    /// - `seed`: Exactly 32 bytes of seed data
    ///
    /// # Security Considerations
    /// - **Seed quality is critical** - use cryptographically secure randomness
    /// - Poor quality seeds reduce security significantly
    /// - Same seed always produces same embedding pattern
    /// - Seed must be stored/transmitted securely
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// // Generate a secure seed (example - use proper CSPRNG in practice)
    /// let seed = [42u8; 32]; // Don't use this in production!
    ///
    /// let config = LSBConfig::random().with_seed(seed);
    /// ```
    ///
    /// # Use Cases
    /// - Testing with reproducible results
    /// - Integration with existing key management
    /// - Custom seed derivation schemes
    /// - Compliance with specific randomness requirements
    pub fn with_seed(mut self, seed: [u8; 32]) -> Self {
        if let EmbeddingPattern::Random(ref mut config) = self.pattern {
            config.seed_source = SeedSource::Manual(seed);
        }
        self
    }

    /// Conditionally set password if provided (CLI helper).
    ///
    /// Convenience method for CLI applications where password might be
    /// optional. Only applies the password if `Some(password)` is provided.
    ///
    /// # Parameters
    /// - `password`: Optional password string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// let user_password: Option<String> = Some("secret".to_string());
    /// let config = LSBConfig::random().with_password_if_some(user_password);
    ///
    /// let no_password: Option<String> = None;
    /// let config = LSBConfig::random().with_password_if_some(no_password);
    /// // This config will still use auto-generated seed
    /// ```
    pub fn with_password_if_some(self, password: Option<String>) -> Self {
        match password {
            Some(pwd) => self.with_password(pwd),
            None => self,
        }
    }

    /// Conditionally set seed if provided (CLI helper).
    ///
    /// Convenience method for CLI applications where seed might be
    /// optional. Only applies the seed if `Some(seed)` is provided.
    ///
    /// # Parameters
    /// - `seed`: Optional 32-byte seed array
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// let user_seed: Option<[u8; 32]> = Some([1u8; 32]);
    /// let config = LSBConfig::random().with_seed_if_some(user_seed);
    ///
    /// let no_seed: Option<[u8; 32]> = None;
    /// let config = LSBConfig::random().with_seed_if_some(no_seed);
    /// // This config will still use auto-generated seed
    /// ```
    pub fn with_seed_if_some(self, seed: Option<[u8; 32]>) -> Self {
        match seed {
            Some(seed) => self.with_seed(seed),
            None => self,
        }
    }

    /// Get the configured bit index.
    ///
    /// Returns the bit position (0-7) that will be modified during
    /// embedding operations.
    ///
    /// # Returns
    /// Bit index where 0 is the least significant bit and 7 is the most significant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// let config = LSBConfig::linear().with_bit_index(2);
    /// assert_eq!(config.bit_index(), 2);
    /// ```
    pub fn bit_index(&self) -> u8 {
        self.bit_index
    }

    /// Get a reference to the embedding pattern configuration.
    ///
    /// Returns the pattern type (Linear or Random) along with its
    /// associated configuration options.
    ///
    /// # Returns
    /// Reference to the EmbeddingPattern enum variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::{LSBConfig, EmbeddingPattern};
    ///
    /// let config = LSBConfig::linear();
    /// match config.pattern() {
    ///     EmbeddingPattern::Linear => println!("Using linear pattern"),
    ///     EmbeddingPattern::Random(_) => println!("Using random pattern"),
    /// }
    /// ```
    pub fn pattern(&self) -> &EmbeddingPattern {
        &self.pattern
    }
}

impl Default for LSBConfig {
    /// Creates a default LSB configuration using random pattern.
    ///
    /// The default provides a good balance of security and convenience:
    /// - Random embedding pattern for better security
    /// - Auto-generated seed (embedded in image header)
    /// - Bit index 0 (least significant bit)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBConfig;
    ///
    /// let config = LSBConfig::default();
    /// // Equivalent to:
    /// let config = LSBConfig::random();
    /// ```
    fn default() -> Self {
        Self::random()
    }
}

// Internal runtime configuration for optimized implementation
#[derive(Debug, Clone)]
pub(crate) struct RuntimeConfig {
    bit_index: u8,
    pattern: RuntimePattern,
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimePattern {
    Linear,
    Random { seed: [u8; 32], embed_seed: bool },
}

impl RuntimeConfig {
    /// Convert from user-facing LSBConfig to internal RuntimeConfig
    fn from_config(config: &LSBConfig) -> Result<Self, PngerError> {
        let pattern = match &config.pattern {
            EmbeddingPattern::Linear => RuntimePattern::Linear,
            EmbeddingPattern::Random(random_config) => {
                let (seed, embed_seed) = match &random_config.seed_source {
                    SeedSource::Auto => {
                        let seed = crypto::CryptoContext::generate_random_seed()
                            .map_err(|e| PngerError::CryptoError(e.to_string()))?;
                        (seed, true)
                    }
                    SeedSource::Password(password) => {
                        let seed = crypto::CryptoContext::derive_seed_from_password(password)
                            .map_err(|e| PngerError::CryptoError(e.to_string()))?;
                        (seed, false)
                    }
                    SeedSource::Manual(seed) => (*seed, false),
                };

                RuntimePattern::Random { seed, embed_seed }
            }
        };

        Ok(RuntimeConfig {
            bit_index: config.bit_index,
            pattern,
        })
    }
}

impl RuntimePattern {
    /// Creates a RuntimePattern by analyzing the image header and user config.
    fn from_header_and_config(
        header: &header::CompleteHeader,
        config: &LSBConfig,
    ) -> Result<Self, PngerError> {
        if header
            .fixed
            .flags
            .contains(header::HeaderFlags::RANDOM_PATTERN)
        {
            // It's a random pattern, so we just need to get the seed.
            let seed = Self::reconstruct_seed(header, config)?;
            let seed_was_embedded = header
                .fixed
                .flags
                .contains(header::HeaderFlags::SEED_EMBEDDED);

            Ok(RuntimePattern::Random {
                seed,
                embed_seed: seed_was_embedded,
            })
        } else {
            // It's a linear pattern.
            Ok(RuntimePattern::Linear)
        }
    }

    /// Reconstructs the seed from the header or user config.
    fn reconstruct_seed(
        header: &header::CompleteHeader,
        config: &LSBConfig,
    ) -> Result<[u8; 32], PngerError> {
        let seed_was_embedded = header
            .fixed
            .flags
            .contains(header::HeaderFlags::SEED_EMBEDDED);

        if seed_was_embedded {
            // Use embedded seed
            header.seed.ok_or_else(|| {
                PngerError::InvalidFormat("Seed embedded flag set but no seed data".to_string())
            })
        } else {
            // Use user-provided seed source
            match &config.pattern {
                EmbeddingPattern::Random(random_config) => match &random_config.seed_source {
                    SeedSource::Password(password) => {
                        crypto::CryptoContext::derive_seed_from_password(password)
                            .map_err(|e| PngerError::CryptoError(e.to_string()))
                    }
                    SeedSource::Manual(seed) => Ok(*seed),
                    SeedSource::Auto => Err(PngerError::InvalidFormat(
                        "Auto seed source but no seed embedded".to_string(),
                    )),
                },
                EmbeddingPattern::Linear => Err(PngerError::InvalidFormat(
                    "Linear pattern expected but random pattern found".to_string(),
                )),
            }
        }
    }
}

/// High-level LSB steganography operations.
///
/// This struct provides the main API for embedding and extracting payloads
/// using LSB steganography. It encapsulates the complexity of header management,
/// pattern selection, and cryptographic operations.
///
/// # Design Philosophy
///
/// The LSBEmbedder uses a stateless design where all configuration is provided
/// through the LSBConfig parameter. This ensures thread safety and makes the
/// API predictable and testable.
///
/// # Examples
///
/// ## Basic Embedding and Extraction
/// ```rust
/// use pnger::strategy::lsb::{LSBEmbedder, LSBConfig};
///
/// let mut image_data = vec![0u8; 1000]; // Your image data
/// let payload = b"Hello, World!";
///
/// // Embed with linear pattern
/// let result = LSBEmbedder::embed(&mut image_data, payload, &LSBConfig::linear());
/// assert!(result.is_ok());
///
/// // Extract the payload
/// let result = LSBEmbedder::extract(&mut image_data, &LSBConfig::linear());
/// assert_eq!(result.unwrap().payload, payload);
/// ```
///
/// ## Security-Focused Usage
/// ```rust
/// use pnger::strategy::lsb::{LSBEmbedder, LSBConfig};
///
/// let mut image_data = vec![0u8; 1000];
/// let payload = b"Secret message";
/// let password = "my_secure_password";
///
/// // Embed with password protection
/// let config = LSBConfig::random().with_password(password.to_string());
/// LSBEmbedder::embed(&mut image_data, payload, &config).unwrap();
///
/// // Extract with same password
/// let config = LSBConfig::random().with_password(password.to_string());
/// let result = LSBEmbedder::extract(&mut image_data, &config).unwrap();
/// assert_eq!(result.payload, payload);
/// ```
pub struct LSBEmbedder;

/// Result of a successful embedding operation.
///
/// Contains metadata about the embedding operation that can be useful
/// for analysis, optimization, or user feedback.
///
/// # Fields
///
/// - `bytes_used`: Total bytes of image data modified (header + payload)
/// - `header_size`: Bytes used for the steganography header
/// - `seed_embedded`: Whether the random seed was stored in the image
///
/// # Examples
///
/// ```rust
/// use pnger::strategy::lsb::{LSBEmbedder, LSBConfig};
///
/// let mut image = vec![0u8; 1000];
/// let result = LSBEmbedder::embed(&mut image, b"test", &LSBConfig::linear()).unwrap();
///
/// println!("Embedded payload using {} bytes", result.bytes_used);
/// println!("Header size: {} bytes", result.header_size);
/// println!("Seed embedded: {}", result.seed_embedded);
/// ```
#[derive(Debug, Clone)]
pub struct EmbedResult {
    /// Total number of image bytes modified during embedding.
    ///
    /// This includes both header bytes and payload bytes. Each bit of
    /// payload data requires modifying one bit in the image, so a 100-byte
    /// payload requires modifying 800 image bytes (plus header overhead).
    pub bytes_used: usize,

    /// Number of bytes used for the steganography header.
    ///
    /// The header contains metadata needed for extraction including:
    /// - Payload size
    /// - Embedding pattern flags
    /// - Random seed (if auto-generated)
    /// - Checksum for integrity verification
    pub header_size: usize,

    /// Whether the random seed was embedded in the image header.
    ///
    /// - `true`: Auto-generated seed stored in image (SeedSource::Auto)
    /// - `false`: Password or manual seed used (no seed storage needed)
    pub seed_embedded: bool,
}

/// Result of a successful extraction operation.
///
/// Contains the extracted payload and metadata about the extraction
/// process that can be useful for verification or analysis.
///
/// # Fields
///
/// - `payload`: The extracted payload data
/// - `header_size`: Size of the steganography header that was read
/// - `seed_was_embedded`: Whether the image contained an embedded seed
///
/// # Examples
///
/// ```rust
/// use pnger::strategy::lsb::{LSBEmbedder, LSBConfig};
///
/// let mut image = vec![0u8; 1000];
///
/// // First embed a payload
/// LSBEmbedder::embed(&mut image, b"test payload", &LSBConfig::linear()).unwrap();
///
/// // Then extract it
/// let result = LSBEmbedder::extract(&mut image, &LSBConfig::linear()).unwrap();
/// println!("Extracted {} bytes", result.payload.len());
/// println!("Header was {} bytes", result.header_size);
/// println!("Had embedded seed: {}", result.seed_was_embedded);
/// ```
#[derive(Debug, Clone)]
pub struct ExtractResult {
    /// The extracted payload data.
    ///
    /// This is the original data that was embedded in the image,
    /// recovered bit-by-bit from the modified pixels.
    pub payload: Vec<u8>,

    /// Size of the steganography header that was read.
    ///
    /// Indicates how many bytes at the beginning of the image
    /// were used for metadata rather than payload storage.
    pub header_size: usize,

    /// Whether the image contained an embedded random seed.
    ///
    /// - `true`: Seed was read from the image header (auto-generated)
    /// - `false`: Seed was derived from password or provided manually
    pub seed_was_embedded: bool,
}

impl LSBEmbedder {
    /// Embed payload into image data using specified LSB configuration.
    ///
    /// This is the primary embedding method that handles all aspects of
    /// LSB steganography including header creation, pattern selection,
    /// and payload embedding.
    ///
    /// # Parameters
    /// - `image_data`: Mutable slice of image pixel data to modify
    /// - `payload`: Payload bytes to embed
    /// - `config`: LSB configuration specifying embedding strategy
    ///
    /// # Returns
    /// `EmbedResult` containing embedding statistics and metadata.
    ///
    /// # Process Overview
    /// 1. **Config validation**: Convert user config to internal runtime config
    /// 2. **Header embedding**: Store metadata needed for extraction
    /// 3. **Payload embedding**: Distribute payload bits according to pattern
    /// 4. **Result generation**: Return statistics about the operation
    ///
    /// # Examples
    ///
    /// ## Linear Embedding
    /// ```rust
    /// use pnger::strategy::lsb::{LSBEmbedder, LSBConfig};
    ///
    /// let mut image = vec![0u8; 1000];
    /// let payload = b"Hello, World!";
    ///
    /// let result = LSBEmbedder::embed(
    ///     &mut image,
    ///     payload,
    ///     &LSBConfig::linear()
    /// ).unwrap();
    ///
    /// println!("Used {} bytes for embedding", result.bytes_used);
    /// ```
    ///
    /// ## Secure Random Embedding
    /// ```rust
    /// use pnger::strategy::lsb::{LSBEmbedder, LSBConfig};
    ///
    /// let mut image = vec![0u8; 1000];
    /// let payload = b"Secret data";
    ///
    /// let config = LSBConfig::random()
    ///     .with_password("my_password".to_string())
    ///     .with_bit_index(1);
    ///     
    /// let result = LSBEmbedder::embed(&mut image, payload, &config).unwrap();
    /// assert!(!result.seed_embedded); // Password-derived, no seed storage
    /// ```
    ///
    /// # Errors
    /// - `PngerError::InsufficientCapacity`: Image too small for payload
    /// - `PngerError::CryptoError`: Seed generation or derivation failed
    /// - `PngerError::InvalidFormat`: Invalid configuration parameters
    pub fn embed(
        image_data: &mut [u8],
        payload: &[u8],
        config: &LSBConfig,
    ) -> Result<EmbedResult, PngerError> {
        let runtime_config = RuntimeConfig::from_config(config)?;

        let header_size = header::HeaderEmbedder::required_size(&runtime_config);
        let seed_embedded = matches!(
            runtime_config.pattern,
            RuntimePattern::Random {
                embed_seed: true,
                ..
            }
        );

        let (header_data, body_data) = image_data.split_at_mut(header_size);

        let header_bytes_used = header::HeaderEmbedder::new(header_data, runtime_config.clone())
            .embed(payload.len() as u32)?;
        BodyEmbedder::new(
            body_data,
            runtime_config.pattern.clone(),
            runtime_config.bit_index,
        )
        .embed_payload(payload)?;

        Ok(EmbedResult {
            bytes_used: header_bytes_used + (payload.len() * 8),
            header_size,
            seed_embedded,
        })
    }

    /// Extract payload from image data using specified LSB configuration.
    ///
    /// This is the primary extraction method that reverses the embedding
    /// process to recover the original payload data from modified image pixels.
    ///
    /// # Parameters
    /// - `image_data`: Mutable slice of image pixel data containing embedded payload
    /// - `config`: LSB configuration matching the one used for embedding
    ///
    /// # Returns
    /// `ExtractResult` containing the extracted payload and metadata.
    ///
    /// # Process Overview
    /// 1. **Header parsing**: Read embedded metadata to understand the payload
    /// 2. **Pattern reconstruction**: Rebuild the embedding pattern using config and header
    /// 3. **Payload extraction**: Collect payload bits according to the pattern
    /// 4. **Result generation**: Return payload and extraction metadata
    ///
    /// # Configuration Matching
    ///
    /// The extraction configuration must match the embedding configuration:
    /// - **Linear patterns**: No additional info needed
    /// - **Random with auto seed**: Uses embedded seed from header
    /// - **Random with password**: Must provide the same password
    /// - **Random with manual seed**: Must provide the same seed
    ///
    /// # Examples
    ///
    /// ## Basic Extraction
    /// ```rust
    /// use pnger::strategy::lsb::{LSBEmbedder, LSBConfig};
    ///
    /// let mut image = vec![0u8; 1000];
    ///
    /// // First embed a payload
    /// LSBEmbedder::embed(&mut image, b"test message", &LSBConfig::linear()).unwrap();
    ///
    /// // Then extract it
    /// let result = LSBEmbedder::extract(&mut image, &LSBConfig::linear()).unwrap();
    /// println!("Extracted: {:?}", String::from_utf8_lossy(&result.payload));
    /// ```
    ///
    /// ## Password-Protected Extraction
    /// ```rust
    /// use pnger::strategy::lsb::{LSBEmbedder, LSBConfig};
    ///
    /// let mut image = vec![0u8; 1000];
    ///
    /// // First embed with password
    /// let config = LSBConfig::random().with_password("same_password".to_string());
    /// LSBEmbedder::embed(&mut image, b"secret", &config).unwrap();
    ///
    /// // Then extract with same password
    /// let config = LSBConfig::random().with_password("same_password".to_string());
    /// let result = LSBEmbedder::extract(&mut image, &config).unwrap();
    /// ```
    ///
    /// # Errors
    /// - `PngerError::InvalidFormat`: Corrupted or missing header
    /// - `PngerError::CryptoError`: Password/seed mismatch or derivation failure
    /// - `PngerError::InsufficientData`: Image smaller than expected payload
    pub fn extract(image_data: &mut [u8], config: &LSBConfig) -> Result<ExtractResult, PngerError> {
        // Phase 1: Read fixed header to get flags
        let fixed_header = header::FixedHeader::read_from_bytes(image_data)?;
        let header_size = fixed_header.calculate_total_header_size();

        // Phase 2: Read complete header with variable data
        let complete_header = header::CompleteHeader::read_from_bytes(&image_data[..header_size])?;
        let seed_was_embedded = complete_header
            .fixed
            .flags
            .contains(header::HeaderFlags::SEED_EMBEDDED);

        // Phase 3: Reconstruct runtime pattern from metadata and config
        let runtime_pattern = RuntimePattern::from_header_and_config(&complete_header, config)?;

        // Phase 4: Extract payload using runtime config
        let body_data = &mut image_data[header_size..];
        let mut body_embedder = BodyEmbedder::new(body_data, runtime_pattern, config.bit_index);
        let payload = body_embedder.extract_payload(complete_header.fixed.payload_size as usize)?;

        Ok(ExtractResult {
            payload,
            header_size,
            seed_was_embedded,
        })
    }

    /// Convenience method for linear pattern embedding.
    ///
    /// Equivalent to calling `embed()` with `LSBConfig::linear()`.
    /// Provides a simpler API when you don't need configuration options.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBEmbedder;
    ///
    /// let mut image = vec![0u8; 1000];
    /// let result = LSBEmbedder::embed_linear(&mut image, b"test").unwrap();
    /// ```
    pub fn embed_linear(image_data: &mut [u8], payload: &[u8]) -> Result<EmbedResult, PngerError> {
        Self::embed(image_data, payload, &LSBConfig::linear())
    }

    /// Convenience method for random pattern embedding with auto-generated seed.
    ///
    /// Equivalent to calling `embed()` with `LSBConfig::random()`.
    /// The seed will be automatically generated and embedded in the image.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBEmbedder;
    ///
    /// let mut image = vec![0u8; 1000];
    /// let result = LSBEmbedder::embed_random(&mut image, b"test").unwrap();
    /// assert!(result.seed_embedded); // Auto seed is embedded
    /// ```
    pub fn embed_random(image_data: &mut [u8], payload: &[u8]) -> Result<EmbedResult, PngerError> {
        Self::embed(image_data, payload, &LSBConfig::random())
    }

    /// Convenience method for password-protected random pattern embedding.
    ///
    /// Uses random pattern with password-derived seed. The password is used
    /// to derive a cryptographic seed, so no seed data is stored in the image.
    ///
    /// # Parameters
    /// - `image_data`: Image data to modify
    /// - `payload`: Data to embed
    /// - `password`: Password for seed derivation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBEmbedder;
    ///
    /// let mut image = vec![0u8; 1000];
    /// let result = LSBEmbedder::embed_with_password(
    ///     &mut image,
    ///     b"secret",
    ///     "my_password"
    /// ).unwrap();
    /// assert!(!result.seed_embedded); // Password-derived, no storage needed
    /// ```
    pub fn embed_with_password(
        image_data: &mut [u8],
        payload: &[u8],
        password: &str,
    ) -> Result<EmbedResult, PngerError> {
        let config = LSBConfig::random().with_password(password.to_string());
        Self::embed(image_data, payload, &config)
    }

    /// Convenience method for linear pattern extraction.
    ///
    /// Equivalent to calling `extract()` with `LSBConfig::linear()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBEmbedder;
    ///
    /// let mut image = vec![0u8; 1000];
    ///
    /// // First embed with linear pattern
    /// LSBEmbedder::embed_linear(&mut image, b"test").unwrap();
    ///
    /// // Then extract
    /// let result = LSBEmbedder::extract_linear(&mut image).unwrap();
    /// ```
    pub fn extract_linear(image_data: &mut [u8]) -> Result<ExtractResult, PngerError> {
        Self::extract(image_data, &LSBConfig::linear())
    }

    /// Convenience method for password-protected extraction.
    ///
    /// Uses random pattern with password-derived seed matching the
    /// password used during embedding.
    ///
    /// # Parameters
    /// - `image_data`: Image data containing embedded payload
    /// - `password`: Password used during embedding
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pnger::strategy::lsb::LSBEmbedder;
    ///
    /// let mut image = vec![0u8; 1000];
    ///
    /// // First embed with password
    /// LSBEmbedder::embed_with_password(&mut image, b"secret", "my_password").unwrap();
    ///
    /// // Then extract with same password
    /// let result = LSBEmbedder::extract_with_password(&mut image, "my_password").unwrap();
    /// ```
    pub fn extract_with_password(
        image_data: &mut [u8],
        password: &str,
    ) -> Result<ExtractResult, PngerError> {
        let config = LSBConfig::random().with_password(password.to_string());
        Self::extract(image_data, &config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsb_config_builder_pattern() {
        // Test linear configuration builder
        let config = LSBConfig::linear().with_bit_index(2);
        assert_eq!(config.bit_index(), 2);
        assert!(matches!(config.pattern(), EmbeddingPattern::Linear));

        // Test random configuration builder
        let config = LSBConfig::random().with_password("test".to_string());
        assert_eq!(config.bit_index(), 0);
        match config.pattern() {
            EmbeddingPattern::Random(random_config) => {
                assert!(matches!(random_config.seed_source, SeedSource::Password(_)));
            }
            _ => panic!("Expected Random pattern"),
        }
    }

    #[test]
    fn test_linear_embed_extract_roundtrip() {
        let mut image_data = vec![0u8; 1000];
        let payload = b"Hello, World!";

        // Test linear embedding
        let result = LSBEmbedder::embed(&mut image_data, payload, &LSBConfig::linear());
        assert!(result.is_ok());
        let embed_result = result.unwrap();
        assert!(!embed_result.seed_embedded);
        assert!(embed_result.header_size > 0);

        // Test linear extraction
        let result = LSBEmbedder::extract(&mut image_data, &LSBConfig::linear());
        assert!(result.is_ok());
        let extract_result = result.unwrap();
        assert_eq!(extract_result.payload, payload);
        assert!(!extract_result.seed_was_embedded);
    }

    #[test]
    fn test_random_auto_seed_roundtrip() {
        let mut image_data = vec![0u8; 1000];
        let payload = b"Hello, World!";

        // Test random embedding with auto seed
        let result = LSBEmbedder::embed(&mut image_data, payload, &LSBConfig::random());
        assert!(result.is_ok());
        let embed_result = result.unwrap();
        assert!(embed_result.seed_embedded); // Auto seed should be embedded

        // Test random extraction with auto seed
        let result = LSBEmbedder::extract(&mut image_data, &LSBConfig::random());
        assert!(result.is_ok());
        let extract_result = result.unwrap();
        assert_eq!(extract_result.payload, payload);
        assert!(extract_result.seed_was_embedded);
    }

    #[test]
    fn test_random_password_roundtrip() {
        let mut image_data = vec![0u8; 1000];
        let payload = b"Hello, World!";
        let password = "test_password";

        // Test random embedding with password
        let config = LSBConfig::random().with_password(password.to_string());
        let result = LSBEmbedder::embed(&mut image_data, payload, &config);
        assert!(result.is_ok());
        let embed_result = result.unwrap();
        assert!(!embed_result.seed_embedded); // Password seed should not be embedded

        // Test random extraction with password
        let config = LSBConfig::random().with_password(password.to_string());
        let result = LSBEmbedder::extract(&mut image_data, &config);
        assert!(result.is_ok());
        let extract_result = result.unwrap();
        assert_eq!(extract_result.payload, payload);
        assert!(!extract_result.seed_was_embedded);
    }

    #[test]
    fn test_convenience_methods() {
        let mut image_data = vec![0u8; 1000];
        let payload = b"Hello, World!";

        // Test linear convenience methods
        let result = LSBEmbedder::embed_linear(&mut image_data, payload);
        assert!(result.is_ok());

        let result = LSBEmbedder::extract_linear(&mut image_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().payload, payload);

        // Test password convenience methods
        let mut image_data2 = vec![0u8; 1000];
        let result = LSBEmbedder::embed_with_password(&mut image_data2, payload, "test");
        assert!(result.is_ok());

        let result = LSBEmbedder::extract_with_password(&mut image_data2, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().payload, payload);
    }

    #[test]
    fn test_config_defaults() {
        // Test that default uses random pattern
        let config = LSBConfig::default();
        assert!(matches!(config.pattern(), EmbeddingPattern::Random(_)));
        assert_eq!(config.bit_index(), 0);
    }

    #[test]
    fn test_conditional_setters() {
        // Test password_if_some helper
        let config = LSBConfig::random().with_password_if_some(Some("test".to_string()));
        match config.pattern() {
            EmbeddingPattern::Random(random_config) => {
                assert!(matches!(random_config.seed_source, SeedSource::Password(_)));
            }
            _ => panic!("Expected Random pattern"),
        }

        // Test that None doesn't change the config
        let config = LSBConfig::random().with_password_if_some(None);
        match config.pattern() {
            EmbeddingPattern::Random(random_config) => {
                assert!(matches!(random_config.seed_source, SeedSource::Auto));
            }
            _ => panic!("Expected Random pattern"),
        }
    }
}
