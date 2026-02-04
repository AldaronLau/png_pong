use std::io::{Read, Write};

use parsenic::{Read as _, Reader};

use super::{Chunk, DecoderError, DecoderResult, EncoderError, EncoderResult};
use crate::{consts, decoder::Parser, encoder::Enc, parsing::Read as _, zlib};

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
            return Err(EncoderError::KeySize(self.key.len()));
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
        let buffer = parse.raw()?;
        let mut reader = Reader::new(&buffer);
        let key = {
            let key = reader.strz()?;
            let key_len = key.len();

            (1..=79)
                .contains(&key_len)
                .then_some(key)
                .ok_or(DecoderError::KeySize(key_len))?
        };
        let _compression_method = {
            let compression_method = reader.u8()?;

            (compression_method == 0)
                .then_some(compression_method)
                .ok_or(DecoderError::CompressionMethod)?
        };
        let ztxt = reader.slice(parse.len() - (key.len() + 2))?;
        let decoded = zlib::decompress(ztxt)?;
        let val = String::from_utf8_lossy(&decoded).into_owned();

        Ok(Chunk::CompressedText(CompressedText { key, val }))
    }
}
