// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use super::{DecoderError, EncoderError};
use crate::{checksum::CrcDecoder, consts};
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
    pub(crate) fn read<R: Read>(
        reader: &mut R,
    ) -> Result<(Self, u32), DecoderError> {
        let mut chunk = CrcDecoder::new(reader, consts::PALETTE);
        let mut palette = Vec::new();
        while let Some(red) = chunk.maybe_u8()? {
            let green = chunk.u8()?;
            let blue = chunk.u8()?;
            palette.push(SRgb8::new(red, green, blue));
        }
        Ok((Palette { palette }, chunk.end()?))
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
