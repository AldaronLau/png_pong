use std::io::Read;

use crate::{
    chunk::{
        Background, Chunk, CompressedText, ImageData, ImageEnd, ImageHeader,
        InternationalText, Palette, Physical, Text, Time, Transparency,
        Unknown,
    },
    consts,
    decode::Result,
    decoder::Parser,
};

/// Iterator over [`Chunk`](struct.Chunk.html)s - Decoder for PNG files.
#[derive(Debug)]
pub struct Chunks<R: Read> {
    /// Decoder
    dec: Parser<R>,
}

impl<R: Read> Chunks<R> {
    /// Create a new encoder.  Will return an error if it's not a PNG file.
    pub(crate) fn new(dec: Parser<R>) -> Self {
        Chunks { dec }
    }

    /// Get the next chunk in the PNG file.
    fn get_next(&mut self) -> Result<Option<Chunk>> {
        // Always start reading at the beginning of the next chunk:
        let name = if let Some(name) = self.dec.prepare()? {
            name
        } else {
            return Ok(None);
        };
        // Choose correct parser for the chunk based on it's name.
        use consts::*;
        let chunk = match name {
            IMAGE_HEADER => ImageHeader::parse(&mut self.dec),
            IMAGE_DATA => ImageData::parse(&mut self.dec),
            IMAGE_END => Ok(ImageEnd::parse()),
            PALETTE => Palette::parse(&mut self.dec),
            BACKGROUND => Background::parse(&mut self.dec),
            ITEXT => InternationalText::parse(&mut self.dec),
            PHYSICAL => Physical::parse(&mut self.dec),
            TEXT => Text::parse(&mut self.dec),
            TIME => Time::parse(&mut self.dec),
            TRANSPARENCY => Transparency::parse(&mut self.dec),
            ZTEXT => CompressedText::parse(&mut self.dec),
            id => Unknown::parse(&mut self.dec, id),
        }?;
        // Check the CRC Checksum at the end of the chunk.
        self.dec.check_crc(&name)?;
        // Return the Chunk
        Ok(Some(chunk))
    }
}

impl<R: Read> Iterator for Chunks<R> {
    type Item = Result<Chunk>;

    fn next(&mut self) -> Option<Self::Item> {
        // Do a swappity
        match self.get_next() {
            Ok(Some(c)) => Some(Ok(c)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
