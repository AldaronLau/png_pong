// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::{
    chunk::{
        Background, Chunk, CompressedText, ImageData, ImageEnd,
        ImageHeader, InternationalText, Palette, Physical, Text, Time,
        Transparency,
    },
    consts,
    decode::{Error, Result},
    decoder::Parser,
};
use std::io::Read;

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
        println!("GOT TNAME {:?}", name);
        // Choose correct parser for the chunk based on it's name.
        use consts::*;
        let chunk = match name {
            IMAGE_HEADER => ImageHeader::parse(&mut self.dec),
            IMAGE_DATA => ImageData::parse(&mut self.dec),
            IMAGE_END => ImageEnd::parse(),
            PALETTE => Palette::parse(&mut self.dec),
            BACKGROUND => Background::parse(&mut self.dec),
            ITEXT => InternationalText::parse(&mut self.dec),
            PHYSICAL => Physical::parse(&mut self.dec),
            TEXT => Text::parse(&mut self.dec),
            TIME => Time::parse(&mut self.dec),
            TRANSPARENCY => Transparency::parse(&mut self.dec),
            ZTEXT => CompressedText::parse(&mut self.dec),
            id => {
                self.dec.unknown_chunk()?;
                self.dec.check_crc(&name)?;
                return Err(Error::UnknownChunkType(id));
            }
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
