use std::io::{Read, Write};

use super::{Chunk, DecoderError, EncoderError};
use crate::{consts, decoder::Parser, encoder::Enc};

/// Non-International Text Chunk Data (tEXt and zTXt)
#[derive(Clone, Debug)]
pub struct Text {
    /// A keyword that gives a short description of what the text in `val`
    /// represents, e.g. Title, Author, Description, or anything else.  Minimum
    /// of 1 character, and maximum 79 characters long.
    pub key: String,
    /// The actual message.  It's discouraged to use a single line length
    /// longer than 79 characters
    pub val: String,
}

impl Text {
    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> Result<Chunk, DecoderError> {
        let key = parse.str()?;
        if key.is_empty() || key.len() > 79 {
            return Err(DecoderError::TextSize(key.len()));
        }
        let val = parse.string(parse.len() - (key.len() + 1))?;

        Ok(Chunk::Text(Text { key, val }))
    }

    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        // Checks
        if self.key.as_bytes().is_empty() || self.val.as_bytes().len() > 79 {
            return Err(EncoderError::TextSize(self.val.as_bytes().len()));
        }

        // 1 Null-terminated string, 1 string
        enc.prepare(self.key.len() + self.val.len() + 1, consts::TEXT)?;
        enc.str(&self.key)?;
        enc.string(&self.val)?;
        enc.write_crc()
    }
}
