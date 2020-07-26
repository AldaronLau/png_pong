// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use super::{Chunk, DecoderError, EncoderError};
use crate::{consts, decoder::Parser};
use pix::rgb::{Rgb, SRgb8};
use std::io::{Read, Write};

/// Palette Chunk Data (PLTE)
#[derive(Clone, Debug)]
#[must_use]
pub struct Palette {
    /// List of colors in the palette.
    pub palette: Vec<SRgb8>,
}

impl Palette {
    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> Result<Chunk, DecoderError> {
        parse.set_palette();
        let mut palette = Vec::new();
        for _ in 0..(parse.len() / 3) {
            let red = parse.u8()?;
            let green = parse.u8()?;
            let blue = parse.u8()?;
            palette.push(SRgb8::new(red, green, blue));
        }
        Ok(Chunk::Palette(Palette { palette }))
    }

    pub(crate) fn write<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), EncoderError> {
        let mut plte = Vec::new();
        for p in self.palette.iter().cloned() {
            super::encode_u8(&mut plte, Rgb::red(p).into())?;
            super::encode_u8(&mut plte, Rgb::green(p).into())?;
            super::encode_u8(&mut plte, Rgb::blue(p).into())?;
        }

        super::encode_chunk(writer, consts::PALETTE, &plte)
    }
}
