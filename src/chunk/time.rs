use std::io::{Read, Write};

use parsenic::{Read as _, Reader, be::Read as _};

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
        let buffer: [u8; 7] = parse.bytes()?;
        let mut reader = Reader::new(&buffer);
        let year = reader.u16()?;
        let month = reader.u8()?;
        let day = reader.u8()?;
        let hour = reader.u8()?;
        let minute = reader.u8()?;
        let second = reader.u8()?;

        reader.end().unwrap();
        Ok(Chunk::Time(Time {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }))
    }
}
