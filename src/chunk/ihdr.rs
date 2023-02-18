use std::io::{Read, Write};

use crate::{
    chunk::Chunk, consts, decode::Error as DecoderError, decoder::Parser,
    encode::Error as EncoderError, encoder::Enc,
};

/// Standard PNG color types.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ColorType {
    /// greyscale: 1, 2, 4, 8, 16 bit
    Grey = 0u8,
    /// RGB: 8, 16 bit
    Rgb = 2,
    /// palette: 1, 2, 4, 8 bit
    Palette = 3,
    /// greyscale with alpha: 8, 16 bit
    GreyAlpha = 4,
    /// RGB with alpha: 8, 16 bit
    Rgba = 6,
}

impl ColorType {
    /// channels * bytes per channel = bytes per pixel
    pub(crate) fn channels(self) -> u8 {
        match self {
            ColorType::Grey | ColorType::Palette => 1,
            ColorType::GreyAlpha => 2,
            ColorType::Rgb => 3,
            ColorType::Rgba => 4,
        }
    }

    /// get the total amount of bits per pixel, based on colortype and bitdepth
    /// in the struct
    pub(crate) fn bpp(self, bit_depth: u8) -> u8 {
        assert!((1..=16).contains(&bit_depth));
        /* bits per pixel is amount of channels * bits per channel */
        let ch = self.channels();
        ch * if ch > 1 {
            if bit_depth == 8 {
                8
            } else {
                16
            }
        } else {
            bit_depth
        }
    }

    /// Error if invalid color type / bit depth combination for PNG.
    pub(crate) fn check_png_color_validity(
        self,
        bd: u8,
    ) -> Result<(), DecoderError> {
        match self {
            ColorType::Grey => {
                if !(bd == 1 || bd == 2 || bd == 4 || bd == 8 || bd == 16) {
                    return Err(DecoderError::ColorMode(self, bd));
                }
            }
            ColorType::Palette => {
                if !(bd == 1 || bd == 2 || bd == 4 || bd == 8) {
                    return Err(DecoderError::ColorMode(self, bd));
                }
            }
            ColorType::Rgb | ColorType::GreyAlpha | ColorType::Rgba => {
                if !(bd == 8 || bd == 16) {
                    return Err(DecoderError::ColorMode(self, bd));
                }
            }
        }
        Ok(())
    }
}

/// Image Header Chunk Data (IHDR)
#[derive(Copy, Clone, Debug)]
pub struct ImageHeader {
    /// Width of the image
    pub width: u32,
    /// Height of the image
    pub height: u32,
    /// The colortype of the image
    pub color_type: ColorType,
    /// How many bits per channel
    pub bit_depth: u8,
    /// True for adam7 interlacing, false for no interlacing.
    pub interlace: bool,
}

impl ImageHeader {
    pub(crate) fn write<W: Write>(
        &self,
        enc: &mut Enc<W>,
    ) -> Result<(), EncoderError> {
        enc.prepare(13, consts::IMAGE_HEADER)?;
        enc.u32(self.width)?;
        enc.u32(self.height)?;
        enc.u8(self.bit_depth)?;
        enc.u8(self.color_type as u8)?;
        enc.u8(0)?;
        enc.u8(0)?;
        enc.u8(self.interlace as u8)?;
        enc.write_crc()
    }

    pub(crate) fn parse<R: Read>(
        parse: &mut Parser<R>,
    ) -> Result<Chunk, DecoderError> {
        // Read file
        let width = parse.u32()?;
        let height = parse.u32()?;
        if width == 0 || height == 0 {
            return Err(DecoderError::ImageDimensions);
        }
        let bit_depth = parse.u8()?;
        if bit_depth == 0 || bit_depth > 16 {
            return Err(DecoderError::BitDepth(bit_depth));
        }
        let color_type = match parse.u8()? {
            0 => ColorType::Grey,
            2 => ColorType::Rgb,
            3 => ColorType::Palette,
            4 => ColorType::GreyAlpha,
            6 => ColorType::Rgba,
            c => return Err(DecoderError::ColorType(c)),
        };
        color_type.check_png_color_validity(bit_depth)?;
        if parse.u8()? != 0 {
            /* error: only compression method 0 is allowed in the
             * specification */
            return Err(DecoderError::CompressionMethod);
        }
        if parse.u8()? != 0 {
            /* error: only filter method 0 is allowed in the specification */
            return Err(DecoderError::FilterMethod);
        }
        let interlace = match parse.u8()? {
            0 => false,
            1 => true,
            _ => return Err(DecoderError::InterlaceMethod),
        };

        Ok(Chunk::ImageHeader(Self {
            width,
            height,
            color_type,
            bit_depth,
            interlace,
        }))
    }

    /// get the total amount of bits per pixel, based on colortype and bitdepth
    /// in the struct
    pub(crate) fn bpp(&self) -> u8 {
        self.color_type.bpp(self.bit_depth) /* 4 or 6 */
    }

    /// Returns the byte size of a raw image buffer with given width, height and
    /// color mode
    pub(crate) fn raw_size(&self) -> usize {
        /* will not overflow for any color type if roughly w * h < 268435455 */
        let bpp = self.bpp() as usize;
        let n = self.width as usize * self.height as usize;
        ((n / 8) * bpp) + ((n & 7) * bpp + 7) / 8
    }
}
