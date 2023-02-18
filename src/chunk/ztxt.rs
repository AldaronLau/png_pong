use std::io::{Read, Write};

use super::{Chunk, DecoderError, DecoderResult, EncoderError, EncoderResult};
use crate::{consts, decoder::Parser, encoder::Enc, zlib};

/// Compressed Text Chunk Data (zTXt)
#[derive(Clone, Debug)]
pub struct CompressedText {
    /// A keyword that gives a short description of what the text in `val`
    /// represents, e.g. Title, Author, Description, or anything else.  Minimum
    /// of 1 character, and maximum 79 characters long.
    pub key: String,
    /// The actual message.  It's discouraged to use a single line length
    /// longer than 79 characters
    pub val: String,
}

impl CompressedText {
    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> EncoderResult<()> {
        // Checks
        if self.key.as_bytes().is_empty() || self.key.as_bytes().len() > 79 {
            return Err(EncoderError::TextSize(self.key.len()));
        }

        // Compress text
        let mut zdata = Vec::new();
        zlib::compress(&mut zdata, self.val.as_bytes(), enc.level());

        // Encode Chunk
        enc.prepare(self.key.len() + 2 + zdata.len(), consts::ZTEXT)?;
        enc.str(&self.key)?;
        enc.u8(0)?; // Compression Method
        enc.raw(&zdata)?;
        enc.write_crc()
    }

    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> DecoderResult<Chunk> {
        let key = parse.str()?;
        if parse.u8()? != 0 {
            return Err(DecoderError::CompressionMethod);
        }
        let ztxt = parse.vec(parse.len() - (key.len() + 2))?;
        let decoded = zlib::decompress(&ztxt)?;
        if key.is_empty() || key.len() > 79 {
            return Err(DecoderError::TextSize(key.len()));
        }
        let val = String::from_utf8_lossy(&decoded).to_string();

        Ok(Chunk::CompressedText(CompressedText { key, val }))
    }
}
