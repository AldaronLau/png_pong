use std::io::{Read, Write};

use parsenic::{Read as _, Reader, be::Read as _};

use super::{Chunk, DecoderError, EncoderError};
use crate::{consts, decoder::Parser, encoder::Enc};

/// Physical dimensions chunk (pHYs)
#[derive(Copy, Clone, Debug)]
pub struct Physical {
    /// Pixels per unit: X dimension
    pub ppu_x: u32,
    /// Pixels per unit: Y dimension
    pub ppu_y: u32,
    /// Unit is `meter` if true, `unknown` if false.
    pub is_meter: bool,
}

impl Physical {
    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        enc.prepare(9, consts::PHYSICAL)?;
        enc.u32(self.ppu_x)?;
        enc.u32(self.ppu_y)?;
        enc.u8(if self.is_meter { 1 } else { 0 })?;
        enc.write_crc()
    }

    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> Result<Chunk, DecoderError> {
        let buffer: [u8; 9] = parse.bytes()?;
        let mut reader = Reader::new(&buffer);
        let ppu_x = reader.u32()?;
        let ppu_y = reader.u32()?;
        let is_meter = match reader.u8()? {
            0 => false,
            1 => true,
            _ => return Err(DecoderError::PhysUnits),
        };

        reader.end().unwrap();
        Ok(Chunk::Physical(Physical {
            ppu_x,
            ppu_y,
            is_meter,
        }))
    }
}
