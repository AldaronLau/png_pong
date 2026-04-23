use std::io::{BufRead, Write};

use parsenic::{Read as _, Reader};

use super::{Chunk, DecoderError, EncoderError};
use crate::{consts, decoder::Parser, encoder::Enc, parsing::Read as _};

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
    pub(crate) fn parse<R: BufRead>(
        parse: &mut Parser<R>,
    ) -> Result<Chunk, DecoderError> {
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
        let val = String::from_utf8_lossy(
            reader.slice(parse.len() - (key.len() + 1))?,
        )
        .into_owned();

        reader.end().unwrap();
        Ok(Chunk::Text(Text { key, val }))
    }

    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        // Checks
        if self.key.is_empty() {
            return Err(EncoderError::KeySize(0));
        }

        // 1 Null-terminated string, 1 string
        enc.prepare(self.key.len() + self.val.len() + 1, consts::TEXT)?;
        enc.str(&self.key)?;
        enc.string(&self.val)?;
        enc.write_crc()
    }
}
