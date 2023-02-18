use std::io::{Read, Write};

use super::{Chunk, DecoderResult, EncoderResult};
use crate::{decoder::Parser, encoder::Enc};

/// An unknown PNG data chunk
#[derive(Clone, Debug)]
pub struct Unknown {
    /// The chunk name
    pub name: [u8; 4],
    /// The chunk data
    pub data: Vec<u8>,
}

impl Unknown {
    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> EncoderResult<()> {
        enc.prepare(self.data.len(), self.name)?;
        enc.raw(&self.data)?;
        enc.write_crc()
    }

    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
        name: [u8; 4],
    ) -> DecoderResult<Chunk> {
        let data = parse.unknown_chunk()?;

        Ok(Chunk::Unknown(Unknown { name, data }))
    }
}
