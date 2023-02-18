use std::io::Write;

use crate::{
    consts,
    encode::{ChunkEnc, Error, FilterStrategy, Result, StepEnc},
};

/// Chunk encoder.
#[derive(Debug)]
pub(crate) struct Enc<W: Write> {
    /// Encoder
    encode: Encoder<W>,
    /// CRC32
    chksum: u32,
}

impl<W: Write> Enc<W> {
    /// Prepare a chunk for writing (reset checksum).
    pub(crate) fn prepare(&mut self, len: usize, name: [u8; 4]) -> Result<()> {
        assert!(len <= consts::MAX_CHUNK_SIZE);
        let len: u32 = len.try_into().unwrap();
        self.encode
            .writer
            .write_all(&len.to_be_bytes())
            .map_err(Error::from)?;
        self.chksum = consts::CRC32_INIT;
        for c in name.iter().cloned() {
            self.u8(c)?;
        }
        Ok(())
    }

    /// Write a u8
    pub(crate) fn u8(&mut self, value: u8) -> Result<()> {
        self.encode
            .writer
            .write_all(&[value])
            .map_err(Error::from)?;
        let index: usize = (self.chksum as u8 ^ value).into();
        self.chksum = consts::CRC32_LOOKUP[index] ^ (self.chksum >> 8);
        Ok(())
    }

    /// Write a u16
    pub(crate) fn u16(&mut self, value: u16) -> Result<()> {
        let bytes = value.to_be_bytes();
        for byte in bytes.iter().cloned() {
            self.u8(byte)?;
        }
        Ok(())
    }

    /// Write a u32
    pub(crate) fn u32(&mut self, value: u32) -> Result<()> {
        let bytes = value.to_be_bytes();
        for byte in bytes.iter().cloned() {
            self.u8(byte)?;
        }
        Ok(())
    }

    /// Write a string
    pub(crate) fn string(&mut self, value: &str) -> Result<()> {
        for byte in value.bytes() {
            self.u8(byte)?;
        }
        Ok(())
    }

    /// Write a null-terminated string
    pub(crate) fn str(&mut self, value: &str) -> Result<()> {
        self.string(value)?;
        self.u8(0)
    }

    /// Write raw data
    pub(crate) fn raw(&mut self, raw: &[u8]) -> Result<()> {
        for byte in raw.iter().cloned() {
            self.u8(byte)?;
        }
        Ok(())
    }

    /// Calculate and write Chunk CRC, ending the chunk.
    pub(crate) fn write_crc(&mut self) -> Result<()> {
        let crc = self.chksum ^ consts::CRC32_INIT;
        self.encode
            .writer
            .write_all(&crc.to_be_bytes())
            .map_err(Error::from)
    }

    /// Get the chosen filter strategy    
    pub(crate) fn filter_strategy(&self) -> Option<FilterStrategy> {
        self.encode.filter_strategy
    }

    /// Get the compression level.    
    pub(crate) fn level(&self) -> u8 {
        self.encode.level
    }

    /// Whether or not interlaced.    
    pub(crate) fn interlace(&self) -> bool {
        self.encode.interlace
    }
}

/// PNG file encoder
///
/// Can be converted into one of two encoders:
/// - [into_step_enc] for high-level [Step]s
/// - [into_chunk_enc] for low-level [Chunk]s
///
/// [into_iter]: struct.Decoder.html#method.into_iter
/// [into_step_enc]: struct.Decoder.html#method.into_step_enc
/// [into_chunk_enc]: struct.Decoder.html#method.into_chunk_enc
/// [Step]: struct.Step.html
/// [Chunk]: struct.Chunk.html
#[derive(Debug)]
pub struct Encoder<W: Write> {
    filter_strategy: Option<FilterStrategy>,
    level: u8,
    interlace: bool,
    writer: W,
}

impl<W: Write> Encoder<W> {
    /// Create a new PNG encoder.
    pub fn new(writer: W) -> Self {
        Encoder {
            writer,
            filter_strategy: None,
            level: 6,
            interlace: false,
        }
    }

    /// Set a specific filter strategy.  If this is never called, than png_pong
    /// attempts to choose the best (compromise speed / compression) filter
    /// strategy.
    pub fn filter_strategy(mut self, strategy: FilterStrategy) -> Self {
        self.filter_strategy = Some(strategy);
        self
    }

    /// Set the compression level (default: 6).  Must be between 0 and 10.
    pub fn compression_level(mut self, level: u8) -> Self {
        assert!(level <= 10);
        self.level = level;
        self
    }

    /// Encode interlaced (default non-interlaced)
    pub fn interlace(mut self) -> Self {
        self.interlace = true;
        self
    }

    /// Convert into a chunk encoder.
    pub fn into_chunk_enc(self) -> ChunkEnc<W> {
        ChunkEnc::new(self.into_enc())
    }

    /// Convert into a step encoder.
    pub fn into_step_enc(self) -> StepEnc<W> {
        StepEnc::new(self.into_chunk_enc())
    }

    fn into_enc(self) -> Enc<W> {
        Enc {
            encode: self,
            chksum: 0,
        }
    }
}
