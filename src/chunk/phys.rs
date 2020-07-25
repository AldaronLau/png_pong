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
use std::io::{Read, Write};

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
        writer: &mut W,
    ) -> Result<(), EncoderError> {
        // 9 bytes
        let mut data = Vec::new();
        super::encode_u32(&mut data, self.ppu_x)?;
        super::encode_u32(&mut data, self.ppu_y)?;
        super::encode_u8(&mut data, if self.is_meter { 1 } else { 0 })?;

        super::encode_chunk(writer, consts::PHYSICAL, &data)
    }

    pub(crate) fn read<R: Read>(
        reader: &mut R,
    ) -> Result<(Self, u32), DecoderError> {
        let mut chunk = CrcDecoder::new(reader, consts::PHYSICAL);

        // 9 bytes
        let ppu_x = chunk.u32()?;
        let ppu_y = chunk.u32()?;
        let is_meter = match chunk.u8()? {
            0 => false,
            1 => true,
            _ => return Err(DecoderError::PhysUnits),
        };

        Ok((
            Physical {
                ppu_x,
                ppu_y,
                is_meter,
            },
            chunk.end()?,
        ))
    }
}
