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
use std::io::{Read, Write};

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
        writer: &mut W,
    ) -> Result<(), EncoderError> {
        let mut data = Vec::new();
        // 7 Bytes
        super::encode_u16(&mut data, self.year)?;
        super::encode_u8(&mut data, self.month)?;
        super::encode_u8(&mut data, self.day)?;
        super::encode_u8(&mut data, self.hour)?;
        super::encode_u8(&mut data, self.minute)?;
        super::encode_u8(&mut data, self.second)?;

        super::encode_chunk(writer, consts::TIME, &data)
    }

    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> Result<Chunk, DecoderError> {
        // 7 Bytes
        let year = parse.u16()?;
        let month = parse.u8()?;
        let day = parse.u8()?;
        let hour = parse.u8()?;
        let minute = parse.u8()?;
        let second = parse.u8()?;

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
