use std::io::{BufRead, Write};

use parsenic::{Read as _, Reader};
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
    pub(crate) fn parse<R: BufRead>(
        parse: &mut Parser<R>,
    ) -> Result<Chunk, DecoderError> {
        parse.set_palette();

        let buffer = parse.raw()?;
        let mut reader = Reader::new(&buffer);
        let palette = (0..(parse.len() / 3))
            .map(|_| -> Result<_, DecoderError> {
                let [r, g, b] = [reader.u8()?, reader.u8()?, reader.u8()?];

                Ok(SRgb8::new(r, g, b))
            })
            .collect::<Result<_, _>>()?;

        reader.end().unwrap();
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
