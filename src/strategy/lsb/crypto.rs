use super::SEED_SIZE;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Random generation error: {0}")]
    GetRandom(#[from] getrandom::Error),
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),
    #[error("Invalid seed length: expected {expected} bytes, got {0}", expected = SEED_SIZE)]
    InvalidSeedLength(usize),
}

/// Crypto mode determines how the seed is generated
#[derive(Debug, Clone)]
pub enum CryptoMode {
    /// Auto-generate random seed (will be embedded in PNG)
    Auto,
    /// Derive seed from password using Argon2 (nothing embedded)
    Password(String),
    /// User provides raw seed directly
    Manual([u8; SEED_SIZE]),
}

impl Default for CryptoMode {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Default, Clone)]
pub struct CryptoParams {
    pub mode: CryptoMode,
}

impl CryptoParams {
    pub fn auto() -> Self {
        Self {
            mode: CryptoMode::Auto,
        }
    }

    pub fn password(password: String) -> Self {
        Self {
            mode: CryptoMode::Password(password),
        }
    }

    pub fn manual(seed: [u8; SEED_SIZE]) -> Self {
        Self {
            mode: CryptoMode::Manual(seed),
        }
    }

    pub fn is_embeddable(&self) -> bool {
        matches!(self.mode, CryptoMode::Auto)
    }
}

#[derive(Debug)]
pub struct CryptoContext {
    pub seed: [u8; SEED_SIZE],
    pub is_embeddable: bool,
}

impl CryptoContext {
    pub const fn new(seed: [u8; SEED_SIZE], is_embeddable: bool) -> Self {
        Self {
            seed,
            is_embeddable,
        }
    }

    fn generate_random_bytes<const N: usize>() -> Result<[u8; N], CryptoError> {
        let mut bytes = [0u8; N];
        getrandom::fill(&mut bytes)?;
        Ok(bytes)
    }

    pub fn generate_random_seed() -> Result<[u8; SEED_SIZE], CryptoError> {
        Self::generate_random_bytes::<SEED_SIZE>()
    }

    /// Derive seed from password using Argon2 with built-in salt
    pub fn derive_seed_from_password(password: &str) -> Result<[u8; SEED_SIZE], CryptoError> {
        use argon2::Argon2;

        // Built-in salt ensures reproducibility without storing salt
        let salt = b"pnger_steganography_salt_v1_____"; // 32 bytes
        let mut seed = [0u8; SEED_SIZE];

        let argon2 = Argon2::default();
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut seed)
            .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

        Ok(seed)
    }

    pub fn from_params(params: CryptoParams) -> Result<Self, CryptoError> {
        let (seed, is_embeddable) = match params.mode {
            CryptoMode::Auto => {
                let seed = Self::generate_random_seed()?;
                (seed, true)
            }
            CryptoMode::Password(password) => {
                let seed = Self::derive_seed_from_password(&password)?;
                (seed, false)
            }
            CryptoMode::Manual(seed) => (seed, false),
        };

        Ok(Self::new(seed, is_embeddable))
    }
}

impl Clone for CryptoContext {
    fn clone(&self) -> Self {
        Self::new(self.seed, self.is_embeddable)
    }
}
