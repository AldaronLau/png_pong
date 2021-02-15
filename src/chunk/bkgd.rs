// PNG Pong
//
// Copyright © 2019-2021 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::io::{Read, Write};

use super::{Chunk, DecoderError, EncoderError};
use crate::{consts, decoder::Parser, encoder::Enc};

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
    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> Result<Chunk, DecoderError> {
        match parse.len() {
            1 => Ok(Chunk::Background(Background::Palette(parse.u8()?))),
            2 => Ok(Chunk::Background(Background::Gray(parse.u16()?))),
            6 => Ok(Chunk::Background(Background::Rgb(
                parse.u16()?,
                parse.u16()?,
                parse.u16()?,
            ))),
            _ => Err(DecoderError::ChunkLength(consts::BACKGROUND)),
        }
    }

    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        use Background::*;
        match *self {
            Palette(v) => {
                enc.prepare(1, consts::BACKGROUND)?;
                enc.u8(v)?;
            }
            Gray(v) => {
                enc.prepare(2, consts::BACKGROUND)?;
                enc.u16(v)?
            }
            Rgb(r, g, b) => {
                enc.prepare(6, consts::BACKGROUND)?;
                enc.u16(r)?;
                enc.u16(g)?;
                enc.u16(b)?;
            }
        }
        enc.write_crc()
    }
}
