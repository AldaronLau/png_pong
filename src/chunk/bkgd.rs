// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::io::{Read, Write};

use super::{DecoderError, EncoderError};
use crate::{checksum::CrcDecoder, consts};

/// Suggested background color chunk (bKGD)
#[derive(Copy, Clone, Debug)]
pub enum Background {
    /// 8-bit palette background index
    Palette(u8),
    /// 1-16 bit gray background value
    Gray(u16),
    /// 1-16 bits for each of Red, Green and Blue
    Rgb(u16, u16, u16),
}

impl Background {
    pub(crate) fn read<R: Read>(
        reader: &mut R,
        chunk_length: u32,
    ) -> Result<(Self, u32), DecoderError> {
        let mut chunk = CrcDecoder::new(reader, consts::BACKGROUND);
        match chunk_length {
            1 => Ok((Background::Palette(chunk.u8()?), chunk.end()?)),
            2 => Ok((Background::Gray(chunk.u16()?), chunk.end()?)),
            6 => Ok((
                Background::Rgb(chunk.u16()?, chunk.u16()?, chunk.u16()?),
                chunk.end()?,
            )),
            _ => Err(DecoderError::ChunkLength(consts::BACKGROUND)),
        }
    }

    pub(crate) fn write<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncoderError> {
        let mut bkgd = Vec::new();
        use Background::*;
        match *self {
            Palette(v) => super::encode_u8(&mut bkgd, v)?,
            Gray(v) => super::encode_u16(&mut bkgd, v)?,
            Rgb(r, g, b) => {
                super::encode_u16(&mut bkgd, r)?;
                super::encode_u16(&mut bkgd, g)?;
                super::encode_u16(&mut bkgd, b)?;
            }
        }
        super::encode_chunk(writer, consts::BACKGROUND, &bkgd)
    }
}
