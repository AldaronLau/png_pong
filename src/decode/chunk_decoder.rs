// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::{
    checksum,
    chunk::{
        self, Background, Chunk, CompressedText, ImageData, ImageEnd,
        ImageHeader, InternationalText, Palette, Physical, Text, Time,
        Transparency,
    },
    consts,
    decode::{Error, Result},
};
use std::io::Read;

/// Iterator over [`Chunk`](struct.Chunk.html)s - Decoder for PNG files.
#[derive(Debug)]
pub struct ChunkDecoder<R: Read> {
    pub(crate) bytes: std::io::Take<R>,
    /// Palette length
    pl: usize,
}

impl<R: Read> ChunkDecoder<R> {
    /// Create a new encoder.  Will return an error if it's not a PNG file.
    pub fn new(r: R) -> Result<Self> {
        let mut r = r.take(8);
        let mut buf = [0u8; 8];
        let bytes_count = match chunk::read(&mut r, &mut buf) {
            Ok(data) => data,
            Err(err) => return Err(err),
        };
        if buf[..bytes_count] != crate::consts::PNG_SIGNATURE {
            return Err(Error::InvalidSignature);
        }
        assert!(r.read_exact(&mut [0]).is_err());

        Ok(ChunkDecoder { bytes: r, pl: 0 })
    }
}

impl<R: Read> Iterator for ChunkDecoder<R> {
    type Item = Result<Chunk>;

    fn next(&mut self) -> Option<Self::Item> {
        // Always start reading at the beginning of the next chunk:
        self.bytes.set_limit(8);
        let mut buf = [0u8; 8];
        let bytes_count = match chunk::read(&mut self.bytes, &mut buf) {
            Ok(data) => data,
            Err(err) => return Some(Err(err)),
        };
        assert!(self.bytes.read_exact(&mut [0]).is_err());
        if bytes_count == 0 {
            return None;
        }
        if bytes_count != 8 {
            return Some(Err(Error::Eof));
        }
        let length = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if length > 2u32.pow(31) {
            return Some(Err(Error::ChunkSize));
        }
        let name = [buf[4], buf[5], buf[6], buf[7]];
        self.bytes.set_limit(length.into());
        //
        use consts::*;
        let (ret, checksum) = match name {
            IMAGE_HEADER => match ImageHeader::read(&mut self.bytes) {
                Ok((ret, crc)) => (Some(Ok(Chunk::ImageHeader(ret))), crc),
                Err(err) => return Some(Err(err)),
            },
            IMAGE_DATA => match ImageData::read(&mut self.bytes) {
                Ok((ret, crc)) => (Some(Ok(Chunk::ImageData(ret))), crc),
                Err(err) => return Some(Err(err)),
            },
            IMAGE_END => match ImageEnd::read(&mut self.bytes) {
                Ok((ret, crc)) => (Some(Ok(Chunk::ImageEnd(ret))), crc),
                Err(err) => return Some(Err(err)),
            },
            PALETTE => match Palette::read(&mut self.bytes) {
                Ok((ret, crc)) => {
                    self.pl = ret.palette.len();
                    (Some(Ok(Chunk::Palette(ret))), crc)
                }
                Err(err) => return Some(Err(err)),
            },
            BACKGROUND => match Background::read(&mut self.bytes, length) {
                Ok((ret, crc)) => (Some(Ok(Chunk::Background(ret))), crc),
                Err(err) => return Some(Err(err)),
            },
            ITEXT => match InternationalText::read(&mut self.bytes) {
                Ok((ret, crc)) => {
                    (Some(Ok(Chunk::InternationalText(ret))), crc)
                }
                Err(err) => return Some(Err(err)),
            },
            PHYSICAL => match Physical::read(&mut self.bytes) {
                Ok((ret, crc)) => (Some(Ok(Chunk::Physical(ret))), crc),
                Err(err) => return Some(Err(err)),
            },
            TEXT => match Text::read(&mut self.bytes) {
                Ok((ret, crc)) => (Some(Ok(Chunk::Text(ret))), crc),
                Err(err) => return Some(Err(err)),
            },
            TIME => match Time::read(&mut self.bytes) {
                Ok((ret, crc)) => (Some(Ok(Chunk::Time(ret))), crc),
                Err(err) => return Some(Err(err)),
            },
            TRANSPARENCY => {
                match Transparency::read(&mut self.bytes, self.pl, length) {
                    Ok((ret, crc)) => (Some(Ok(Chunk::Transparency(ret))), crc),
                    Err(err) => return Some(Err(err)),
                }
            }
            ZTEXT => match CompressedText::read(&mut self.bytes) {
                Ok((ret, crc)) => (Some(Ok(Chunk::CompressedText(ret))), crc),
                Err(err) => return Some(Err(err)),
            },
            id => {
                let mut chunk = checksum::CrcDecoder::new(&mut self.bytes, id);
                while let Some(_byte) = match chunk.maybe_u8() {
                    Ok(byte) => byte,
                    Err(err) => return Some(Err(err)),
                } {}

                (
                    Some(Err(Error::UnknownChunkType(id))),
                    match chunk.end() {
                        Ok(crc) => crc,
                        Err(err) => return Some(Err(err)),
                    },
                )
            }
        };

        // Check CRC
        self.bytes.set_limit(4);
        let mut crc = [0u8; 4];
        match self.bytes.read_exact(&mut crc) {
            Ok(_) => {}
            Err(err) => return Some(Err(Error::from(err))),
        }
        if u32::from_be_bytes(crc) != checksum {
            return Some(Err(Error::Crc32(name)));
        }
        assert!(self.bytes.read_exact(&mut [0]).is_err());

        // Return Chunk
        ret
    }
}
