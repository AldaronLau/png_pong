use std::io::{Read, Write};

use super::{Chunk, DecoderError, EncoderError};
use crate::{consts, decoder::Parser, encoder::Enc};

/// Time chunk (tIME)
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)] // self-explanatory
pub struct Time {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Time {
    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        // 7 Bytes
        enc.prepare(7, consts::TIME)?;
        enc.u16(self.year)?;
        enc.u8(self.month)?;
        enc.u8(self.day)?;
        enc.u8(self.hour)?;
        enc.u8(self.minute)?;
        enc.u8(self.second)?;
        enc.write_crc()
    }

    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> Result<Chunk, DecoderError> {
        // 7 Bytes
        Ok(Chunk::Time(Time {
            year: parse.u16()?,
            month: parse.u8()?,
            day: parse.u8()?,
            hour: parse.u8()?,
            minute: parse.u8()?,
            second: parse.u8()?,
        }))
    }
}
