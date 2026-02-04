use std::io::{Read, Write};

use parsenic::{Read as _, Reader, be::Read as _};

use super::{Chunk, DecoderError, DecoderResult, EncoderResult};
use crate::{consts, decoder::Parser, encoder::Enc};

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

    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> DecoderResult<Chunk> {
        if parse.has_palette() {
            // Palette
            let apal = parse.raw()?;
            Ok(Chunk::Transparency(Transparency::Palette(apal)))
        } else {
            // Gray or RGB
            match parse.len() {
                2 => {
                    let buffer: [u8; 2] = parse.bytes()?;
                    let mut reader = Reader::new(&buffer);
                    let value = reader.u16()?;

                    reader.end().unwrap();
                    Ok(Chunk::Transparency(Transparency::GrayKey(value)))
                }
                6 => {
                    let buffer: [u8; 6] = parse.bytes()?;
                    let mut reader = Reader::new(&buffer);
                    let [r, g, b] =
                        [reader.u16()?, reader.u16()?, reader.u16()?];

                    reader.end().unwrap();
                    Ok(Chunk::Transparency(Transparency::RgbKey(r, g, b)))
                }
                _ => Err(DecoderError::ChunkLength(consts::TRANSPARENCY)),
            }
        }
    }

    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> EncoderResult<()> {
        use Transparency::*;
        match self {
            Palette(plte) => {
                enc.prepare(plte.len(), consts::TRANSPARENCY)?;
                for alpha in plte.iter().cloned() {
                    enc.u8(alpha)?;
                }
            }
            RgbKey(red, green, blue) => {
                enc.prepare(6, consts::TRANSPARENCY)?;
                enc.u16(*red)?;
                enc.u16(*green)?;
                enc.u16(*blue)?;
            }
            GrayKey(key) => {
                enc.prepare(2, consts::TRANSPARENCY)?;
                enc.u16(*key)?
            }
        }
        enc.write_crc()
    }
}
