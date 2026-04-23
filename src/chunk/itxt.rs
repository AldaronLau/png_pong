use std::io::{BufRead, Write};

use parsenic::{Read as _, Reader};

use super::Chunk;
use crate::{
    consts, decode::Error as DecoderError, decoder::Parser,
    encode::Error as EncoderError, encoder::Enc, parsing::Read as _, zlib,
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
    /// The actual message.  It's discouraged to use a single line length
    /// longer than 79 characters
    pub val: String,
    /// If the text should be compressed
    pub compressed: bool,
}

impl InternationalText {
    /* international text chunk (iTXt) */
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
        let compressed = match reader.u8()? {
            0 => false,
            1 => true,
            // FIXME: More specific error
            _ => return Err(DecoderError::CompressionMethod),
        };
        let _compression_method = {
            let compression_method = reader.u8()?;

            (compression_method == 0)
                .then_some(compression_method)
                .ok_or(DecoderError::CompressionMethod)?
        };
        let langtag = reader.strz()?;
        let transkey = reader.strz()?;
        let data = reader
            .slice(
                parse.len() - (key.len() + langtag.len() + transkey.len() + 5),
            )?
            .to_vec();
        let val = if compressed {
            String::from_utf8_lossy(&zlib::decompress(&data)?).to_string()
        } else {
            String::from_utf8_lossy(&data).to_string()
        };

        reader.end().unwrap();
        Ok(Chunk::InternationalText(InternationalText {
            key,
            langtag,
            transkey,
            val,
            compressed,
        }))
    }

    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        // Checks
        let k_len = self.key.len();
        if !(1..=79).contains(&k_len) {
            return Err(EncoderError::KeySize(k_len));
        }

        // Maybe compress
        let zdata = if self.compressed {
            let mut data = Vec::new();
            zlib::compress(&mut data, self.val.as_bytes(), enc.level());
            Some(data)
        } else {
            None
        };

        // Encode
        let len = if let Some(ref zdata) = zdata {
            zdata.len()
        } else {
            self.val.len()
        };
        enc.prepare(
            self.key.len() + self.langtag.len() + self.transkey.len() + len + 5,
            consts::ITEXT,
        )?;
        enc.str(&self.key)?;
        enc.u8(self.compressed as u8)?;
        enc.u8(0)?;
        enc.str(&self.langtag)?;
        enc.str(&self.transkey)?;
        if let Some(zdata) = zdata {
            enc.raw(&zdata)?;
        } else {
            enc.raw(self.val.as_bytes())?;
        }
        enc.write_crc()
    }
}
