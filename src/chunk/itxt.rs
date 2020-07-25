// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::io::{Read, Write};

use crate::{
    checksum::CrcDecoder, consts, decode::Error as DecoderError,
    encode::Error as EncoderError, zlib,
};

/// International Text Chunk Data (iTXt)
#[derive(Clone, Debug)]
pub struct InternationalText {
    /// A keyword that gives a short description of what the text in `val`
    /// represents, e.g. Title, Author, Description, or anything else.  Minimum
    /// of 1 character, and maximum 79 characters long.
    pub key: String,
    /// Additional string "langtag"
    pub langtag: String,
    /// Additional string "transkey"
    pub transkey: String,
    /// The actual message.  It's discouraged to use a single line length longer
    /// than 79 characters
    pub val: String,
    /// If the text should be compressed
    pub compressed: bool,
}

impl InternationalText {
    /*international text chunk (iTXt)*/
    pub(crate) fn read<R: Read>(
        reader: &mut R,
    ) -> Result<(Self, u32), DecoderError> {
        let mut chunk = CrcDecoder::new(reader, consts::ITEXT);

        let key = chunk.utf8z()?;
        if key.is_empty() || key.len() > 79 {
            return Err(DecoderError::TextSize(key.len()));
        }
        let compressed = chunk.u8()? != 0;
        if chunk.u8()? != 0 {
            return Err(DecoderError::CompressionMethod);
        }
        let langtag = chunk.utf8z()?;
        let transkey = chunk.utf8z()?;
        let data = chunk.vec_eof()?;

        let val = if compressed {
            String::from_utf8_lossy(&zlib::decompress(&data)?).to_string()
        } else {
            String::from_utf8_lossy(&data).to_string()
        };
        Ok((
            InternationalText {
                key,
                langtag,
                transkey,
                val,
                compressed,
            },
            chunk.end()?,
        ))
    }

    pub(crate) fn write<W: Write>(
        &self,
        writer: &mut W,
        level: u8,
    ) -> Result<(), EncoderError> {
        let k_len = self.key.len();
        if k_len < 1 || k_len > 79 {
            return Err(EncoderError::TextSize(k_len));
        }
        let mut data = Vec::new();
        data.write_all(self.key.as_bytes())
            .map_err(EncoderError::Io)?;
        super::encode_u8(&mut data, 0)?;
        super::encode_u8(&mut data, self.compressed as u8)?;
        super::encode_u8(&mut data, 0)?;
        data.write_all(self.langtag.as_bytes())
            .map_err(EncoderError::Io)?;
        super::encode_u8(&mut data, 0)?;
        data.write_all(self.transkey.as_bytes())
            .map_err(EncoderError::Io)?;
        super::encode_u8(&mut data, 0)?;
        if self.compressed {
            zlib::compress(&mut data, self.val.as_bytes(), level);
        } else {
            data.write_all(self.val.as_bytes())
                .map_err(EncoderError::Io)?;
        }

        super::encode_chunk(writer, *b"iTXt", &data)
    }
}
