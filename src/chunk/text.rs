// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use super::{DecoderError, EncoderError};
use crate::{checksum::CrcDecoder, consts};
use std::io::{Read, Write};

/// Non-International Text Chunk Data (tEXt and zTXt)
#[derive(Clone, Debug)]
pub struct Text {
    /// A keyword that gives a short description of what the text in `val`
    /// represents, e.g. Title, Author, Description, or anything else.  Minimum
    /// of 1 character, and maximum 79 characters long.
    pub key: String,
    /// The actual message.  It's discouraged to use a single line length longer
    /// than 79 characters
    pub val: String,
}

impl Text {
    pub(crate) fn read<R: Read>(
        reader: &mut R,
    ) -> Result<(Self, u32), DecoderError> {
        let mut chunk = CrcDecoder::new(reader, consts::TEXT);
        let key = chunk.utf8z()?;
        if key.is_empty() || key.len() > 79 {
            return Err(DecoderError::TextSize(key.len()));
        }
        let val = chunk.utf8z()?;

        Ok((Text { key, val }, chunk.end()?))
    }

    pub(crate) fn write<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncoderError> {
        if self.key.as_bytes().is_empty() || self.val.as_bytes().len() > 79 {
            return Err(EncoderError::TextSize(self.val.as_bytes().len()));
        }
        let mut text = Vec::new();
        text.write_all(self.key.as_bytes())
            .map_err(EncoderError::Io)?;
        super::encode_u8(&mut text, 0)?;
        text.write_all(self.val.as_bytes())
            .map_err(EncoderError::Io)?;

        super::encode_chunk(writer, consts::TEXT, &text)
    }
}
