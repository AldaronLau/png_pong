use std::io::{Read, Write};

use crate::{
    chunk::Chunk, consts, decode::Result as DecoderResult, decoder::Parser,
    encode::Error as EncoderError, encoder::Enc, zlib,
};

/// Image Data Chunk Data (IDAT)
#[derive(Debug)]
pub struct ImageData {
    /// Part of a compressed ZLIB stream
    pub data: Vec<u8>,
}

impl ImageData {
    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> DecoderResult<Chunk> {
        let data = parse.raw()?;
        Ok(Chunk::ImageData(ImageData { data }))
    }

    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        // FIXME: Should already be compressed.
        let mut zlib = Vec::new();
        zlib::compress(&mut zlib, self.data.as_slice(), enc.level());

        //
        enc.prepare(zlib.len(), consts::IMAGE_DATA)?;
        enc.raw(&zlib)?;
        enc.write_crc()
    }

    /// Construct from raw uncompressed image data.
    pub fn with_data(data: Vec<u8>) -> ImageData {
        ImageData { data }
    }

    /// Get the image data
    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }
}
