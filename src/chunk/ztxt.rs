// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::io::{Read, Write};

use super::{DecoderError, DecoderResult, EncoderError, EncoderResult};
use crate::{checksum, consts, zlib};

/// Compressed Text Chunk Data (zTXt)
#[derive(Clone, Debug)]
pub struct CompressedText {
    /// A keyword that gives a short description of what the text in `val`
    /// represents, e.g. Title, Author, Description, or anything else.  Minimum
    /// of 1 character, and maximum 79 characters long.
    pub key: String,
    /// The actual message.  It's discouraged to use a single line length longer
    /// than 79 characters
    pub val: String,
}

impl CompressedText {
    pub(crate) fn write<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
    ) -> EncoderResult<()> {
        if self.key.as_bytes().is_empty() || self.key.as_bytes().len() > 79 {
            return Err(EncoderError::TextSize(self.key.len()));
        }
        let mut data = Vec::new();
        data.write_all(self.key.as_bytes())
            .map_err(EncoderError::Io)?;
        super::encode_u8(writer, 0)?; // Null termination
        super::encode_u8(writer, 0)?; // Compression Method
        zlib::compress(&mut data, self.val.as_bytes(), level);

        super::encode_chunk(writer, consts::ZTEXT, &data)?;
        Ok(())
    }

    pub(crate) fn read<R: Read>(reader: &mut R) -> DecoderResult<(Self, u32)> {
        let mut chunk = checksum::CrcDecoder::new(reader, consts::ZTEXT);

        let key = chunk.utf8z()?;
        if chunk.u8()? != 0 {
            return Err(DecoderError::CompressionMethod);
        }
        let ztxt = chunk.vec_eof()?;
        let decoded = zlib::decompress(&ztxt)?;
        if key.is_empty() || key.len() > 79 {
            return Err(DecoderError::TextSize(key.len()));
        }
        let val = String::from_utf8_lossy(&decoded).to_string();

        Ok((CompressedText { key, val }, chunk.end()?))
    }
}
