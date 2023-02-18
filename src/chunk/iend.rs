use std::io::Write;

use super::{Chunk, EncoderError};
use crate::{consts, encoder::Enc};

/// Image End Chunk Data (IEND)
#[derive(Copy, Clone, Debug)]
pub struct ImageEnd;

impl ImageEnd {
    pub(crate) fn parse() -> Chunk {
        Chunk::ImageEnd(ImageEnd)
    }

    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        enc.prepare(0, consts::IMAGE_END)?;
        enc.write_crc()
    }
}
