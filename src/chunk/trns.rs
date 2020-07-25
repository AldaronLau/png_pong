// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::io::{Read, Write};

use super::{DecoderError, DecoderResult, EncoderResult};
use crate::{checksum, consts};

/// Alpha Palette Chunk Data (tRNS)
#[derive(Debug, Clone, PartialEq)]
#[allow(variant_size_differences)]
#[must_use]
pub enum Transparency {
    /// Alpha values for the first `alpha.len()` entries in palette.
    Palette(Vec<u8>),
    /// What RGB value should be replaced with a transparent pixel
    RgbKey(u16, u16, u16),
    /// What gray value should be replaced with a transparent pixel
    GrayKey(u16),
}

impl Transparency {
    /// Get the length of a palette, panicking if transparent key
    pub(crate) fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Get the length of a palette, panicking if transparent key
    pub(crate) fn as_slice(&self) -> &[u8] {
        use Transparency::*;
        match self {
            Palette(p) => p.as_slice(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn read<R: Read>(
        reader: &mut R,
        palette_len: usize,
        chunk_length: u32,
    ) -> DecoderResult<(Self, u32)> {
        let mut chunk = checksum::CrcDecoder::new(reader, consts::TRANSPARENCY);

        if palette_len == 0 {
            // Gray or RGB
            match chunk_length {
                2 => Ok((Transparency::GrayKey(chunk.u16()?), chunk.end()?)),
                6 => Ok((
                    Transparency::RgbKey(
                        chunk.u16()?,
                        chunk.u16()?,
                        chunk.u16()?,
                    ),
                    chunk.end()?,
                )),
                _ => Err(DecoderError::ChunkLength(consts::TRANSPARENCY)),
            }
        } else {
            // Palette
            let apal = chunk.vec_eof()?;
            if apal.len() > palette_len {
                // Alpha palette can't be larger than the palette
                return Err(DecoderError::AlphaPaletteLen);
            }
            Ok((Transparency::Palette(apal), chunk.end()?))
        }
    }

    pub(crate) fn write<W: Write>(&self, writer: &mut W) -> EncoderResult<()> {
        use Transparency::*;
        let mut trns = Vec::new();
        match self {
            Palette(plte) => {
                for alpha in plte.iter().cloned() {
                    super::encode_u8(&mut trns, alpha)?;
                }
            }
            RgbKey(red, green, blue) => {
                super::encode_u16(&mut trns, *red)?;
                super::encode_u16(&mut trns, *green)?;
                super::encode_u16(&mut trns, *blue)?;
            }
            GrayKey(key) => super::encode_u16(&mut trns, *key)?,
        }
        super::encode_chunk(writer, consts::TRANSPARENCY, &trns)
    }
}
