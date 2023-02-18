use std::io::{Read, Write};

use pix::rgb::{Rgb, SRgb8};

use super::{Chunk, DecoderError, EncoderError};
use crate::{consts, decoder::Parser, encoder::Enc};

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
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        enc.prepare(self.palette.len() * 3, consts::PALETTE)?;
        for p in self.palette.iter().cloned() {
            enc.u8(Rgb::red(p).into())?;
            enc.u8(Rgb::green(p).into())?;
            enc.u8(Rgb::blue(p).into())?;
        }
        enc.write_crc()
    }
}
