use binrw::BinRead;
use crc32fast::Hasher;
use std::io::{Cursor, Read, Write};
use thiserror::Error;

use crate::{
    strategy::lsb::{RuntimeConfig, RuntimePattern, SEED_SIZE},
    PayloadSize, PngerError,
};

#[derive(Debug, Error)]
pub(super) enum HeaderError {
    #[error("Insufficient space: need {0} bytes, have {1}")]
    InsufficientSpace(usize, usize),

    #[error("Insufficient data for header")]
    InsufficientData,

    #[error("Invalid magic number")]
    InvalidMagic,

    #[error("CRC mismatch: expected {expected:08x}, found {found:08x}")]
    CrcMismatch { expected: u32, found: u32 },

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u8),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<HeaderError> for PngerError {
    fn from(err: HeaderError) -> Self {
        match err {
            HeaderError::InsufficientSpace(needed, have) => PngerError::PayloadError {
                message: format!("Header needs {needed} bytes, have {have}"),
            },
            HeaderError::InsufficientData => {
                PngerError::InvalidFormat("Insufficient header data".to_string())
            }
            HeaderError::InvalidMagic => {
                PngerError::InvalidFormat("Invalid header magic".to_string())
            }
            HeaderError::CrcMismatch { expected, found } => PngerError::InvalidFormat(format!(
                "Header CRC mismatch: expected {expected:08x}, found {found:08x}"
            )),
            HeaderError::UnsupportedVersion(v) => {
                PngerError::InvalidFormat(format!("Unsupported header version: {v}"))
            }
            HeaderError::Io(io_err) => PngerError::FileIo(io_err),
        }
    }
}

// Header constants
const MAGIC: &[u8; 4] = b"PNGR";
const VERSION: u8 = 1;

// Header field sizes
const MAGIC_SIZE: usize = 4;
const VERSION_SIZE: usize = 1;
const FLAGS_SIZE: usize = 1;
const PAYLOAD_SIZE_SIZE: usize = 4;
const CRC32_SIZE: usize = 4;

// Fixed header size (always present)
const FIXED_HEADER_SIZE: usize =
    MAGIC_SIZE + VERSION_SIZE + FLAGS_SIZE + PAYLOAD_SIZE_SIZE + CRC32_SIZE;

// Header flags (simplified)
#[derive(Debug, Clone, Copy, PartialEq, Eq, BinRead)]
pub struct HeaderFlags(u8);

bitflags::bitflags! {
    impl HeaderFlags: u8 {
        const RANDOM_PATTERN = 0b0000_0001;  // 0=Linear, 1=Random
        const SEED_EMBEDDED = 0b0000_0010;   // 1=Seed is embedded in header
    }
}

// Fixed header structure
#[derive(Debug, BinRead)]
#[br(big)]
#[br(magic = b"PNGR")]
pub struct FixedHeader {
    #[br(assert(version == VERSION))]
    pub version: u8,
    pub flags: HeaderFlags,
    pub payload_size: PayloadSize,
    pub crc32: u32,
}

impl FixedHeader {
    pub fn read_from_bytes(data: &[u8]) -> Result<Self, HeaderError> {
        if data.len() < FIXED_HEADER_SIZE {
            return Err(HeaderError::InsufficientData);
        }

        let mut cursor = Cursor::new(data);
        let header = FixedHeader::read_be(&mut cursor).map_err(|e| match e {
            binrw::Error::AssertFail { .. } => HeaderError::UnsupportedVersion(VERSION),
            binrw::Error::BadMagic { .. } => HeaderError::InvalidMagic,
            binrw::Error::Io(io_err) => HeaderError::Io(io_err),
            _ => HeaderError::InsufficientData,
        })?;

        header.validate()?;
        Ok(header)
    }

    pub const fn calculate_total_header_size(&self) -> usize {
        FIXED_HEADER_SIZE
            + if self.flags.contains(HeaderFlags::SEED_EMBEDDED) {
                SEED_SIZE
            } else {
                0
            }
    }

    fn prepare_crc_data(&self) -> [u8; 6] {
        let mut data = [0u8; VERSION_SIZE + FLAGS_SIZE + PAYLOAD_SIZE_SIZE];
        data[0] = self.version;
        data[1] = self.flags.bits();
        data[2..6].copy_from_slice(&self.payload_size.to_be_bytes());
        data
    }

    fn calculate_crc(&self) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(&self.prepare_crc_data());
        hasher.finalize()
    }

    fn validate(&self) -> Result<(), HeaderError> {
        if self.crc32 != self.calculate_crc() {
            return Err(HeaderError::CrcMismatch {
                expected: self.calculate_crc(),
                found: self.crc32,
            });
        }
        Ok(())
    }
}

// Complete header with optional seed
#[derive(Debug)]
pub struct CompleteHeader {
    pub fixed: FixedHeader,
    pub seed: Option<[u8; 32]>,
}

impl CompleteHeader {
    pub fn read_from_bytes(data: &[u8]) -> Result<Self, HeaderError> {
        if data.len() < FIXED_HEADER_SIZE {
            return Err(HeaderError::InsufficientData);
        }

        let mut cursor = Cursor::new(data);
        let fixed = FixedHeader::read_be(&mut cursor).map_err(|e| match e {
            binrw::Error::AssertFail { .. } => HeaderError::UnsupportedVersion(VERSION),
            binrw::Error::BadMagic { .. } => HeaderError::InvalidMagic,
            binrw::Error::Io(io_err) => HeaderError::Io(io_err),
            _ => HeaderError::InsufficientData,
        })?;

        fixed.validate()?;

        // Read seed if present
        let seed = if fixed.flags.contains(HeaderFlags::SEED_EMBEDDED) {
            let required_pos = (cursor.position() as usize)
                .checked_add(SEED_SIZE)
                .ok_or(HeaderError::InsufficientData)?;
            if data.len() < required_pos {
                return Err(HeaderError::InsufficientData);
            }
            let mut seed_bytes = [0u8; SEED_SIZE];
            cursor.read_exact(&mut seed_bytes)?;
            Some(seed_bytes)
        } else {
            None
        };

        Ok(Self { fixed, seed })
    }

    pub const fn header_size(&self) -> usize {
        FIXED_HEADER_SIZE + if self.seed.is_some() { SEED_SIZE } else { 0 }
    }
}

// Header embedder for writing headers
pub(super) struct HeaderEmbedder<'a> {
    bytes: &'a mut [u8],
    config: RuntimeConfig,
}

impl<'a> HeaderEmbedder<'a> {
    pub const fn new(bytes: &'a mut [u8], config: RuntimeConfig) -> Self {
        Self { bytes, config }
    }

    pub fn embed(&mut self, payload_size: u32) -> Result<usize, HeaderError> {
        let header = self.build_header(payload_size);
        let required_size = header.header_size();

        if self.bytes.len() < required_size {
            return Err(HeaderError::InsufficientSpace(
                required_size,
                self.bytes.len(),
            ));
        }

        self.write_header(&header)
    }

    fn build_header(&self, payload_size: u32) -> CompleteHeader {
        let mut flags = HeaderFlags::empty();
        let mut embedded_seed = None;

        if let RuntimePattern::Random { seed, embed_seed } = &self.config.pattern {
            flags |= HeaderFlags::RANDOM_PATTERN;
            if *embed_seed {
                flags |= HeaderFlags::SEED_EMBEDDED;
                embedded_seed = Some(*seed);
            }
        }

        let mut fixed = FixedHeader {
            version: VERSION,
            flags,
            payload_size,
            crc32: 0,
        };
        fixed.crc32 = fixed.calculate_crc();

        CompleteHeader {
            fixed,
            seed: embedded_seed,
        }
    }

    fn write_header(&mut self, header: &CompleteHeader) -> Result<usize, HeaderError> {
        let mut cursor = Cursor::new(&mut self.bytes[..]);

        // Write fixed header
        cursor.write_all(MAGIC)?;
        cursor.write_all(&[header.fixed.version])?;
        cursor.write_all(&[header.fixed.flags.bits()])?;
        cursor.write_all(&header.fixed.payload_size.to_be_bytes())?;
        cursor.write_all(&header.fixed.crc32.to_be_bytes())?;

        // Write seed if present
        if let Some(seed) = &header.seed {
            cursor.write_all(seed)?;
        }

        Ok(cursor.position() as usize)
    }

    pub const fn required_size(config: &RuntimeConfig) -> usize {
        FIXED_HEADER_SIZE
            + if matches!(
                config.pattern,
                RuntimePattern::Random {
                    embed_seed: true,
                    ..
                }
            ) {
                SEED_SIZE
            } else {
                0
            }
    }
}
