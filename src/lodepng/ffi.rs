// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
// Copyright © 2014-2017 Kornel Lesiński
// Copyright © 2005-2016 Lode Vandevenne
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/*use crate::huffman;
use std::ffi::CStr;
use std::mem;
*/

use std::fmt;
use std::os::raw::*;

use crate::lodepng::rustimpl::*;

use crate::chunk::{ITextChunk, TextChunk};
use pix::Rgba8;

/// Type for `decode`, `encode`, etc. Same as standard PNG color types.
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ColorType {
    /// greyscale: 1, 2, 4, 8, 16 bit
    Grey = 0,
    /// RGB: 8, 16 bit
    Rgb = 2,
    /// palette: 1, 2, 4, 8 bit
    Palette = 3,
    /// greyscale with alpha: 8, 16 bit
    GreyAlpha = 4,
    /// RGB with alpha: 8, 16 bit
    Rgba = 6,

    /// Not PNG standard, for internal use only. BGRA with alpha, 8 bit
    Bgra = 6 | 64,
    /// Not PNG standard, for internal use only. BGR no alpha, 8 bit
    Bgr = 2 | 64,
    /// Not PNG standard, for internal use only. BGR no alpha, padded, 8 bit
    Bgrx = 3 | 64,
}

/// Color mode of an image. Contains all information required to decode the pixel
/// bits to RGBA colors. This information is the same as used in the PNG file
/// format, and is used both for PNG and raw image data in LodePNG.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct ColorMode {
    /// color type, see PNG standard
    pub(crate) colortype: ColorType,
    /// bits per sample, see PNG standard
    pub(crate) bitdepth: u32,

    /// palette (`PLTE` and `tRNS`)
    /// Dynamically allocated with the colors of the palette, including alpha.
    /// When encoding a PNG, to store your colors in the palette of the ColorMode, first use
    /// lodepng_palette_clear, then for each color use lodepng_palette_add.
    /// If you encode an image without alpha with palette, don't forget to put value 255 in each A byte of the palette.
    ///
    /// When decoding, by default you can ignore this palette, since LodePNG already
    /// fills the palette colors in the pixels of the raw RGBA output.
    ///
    /// The palette is only supported for color type 3.
    pub(crate) palette: Vec<pix::SRgba8>,

    /// transparent color key (`tRNS`)
    ///
    /// This color uses the same bit depth as the bitdepth value in this struct, which can be 1-bit to 16-bit.
    /// For greyscale PNGs, r, g and b will all 3 be set to the same.
    ///
    /// When decoding, by default you can ignore this information, since LodePNG sets
    /// pixels with this key to transparent already in the raw RGBA output.
    ///
    /// The color key is only supported for color types 0 and 2.
    pub(crate) key: Option<(u32, u32, u32)>,
}

#[derive(Clone, Debug)]
pub(crate) struct DecompressSettings {
    pub(crate) check_adler32: bool,
}

/// Settings for zlib compression. Tweaking these settings tweaks the balance between speed and compression ratio.
#[repr(C)]
#[derive(Clone)]
pub(crate) struct CompressSettings {
    /// the block type for LZ (0, 1, 2 or 3, see zlib standard). Should be 2 for proper compression.
    pub(crate) btype: u32,
    /// whether or not to use LZ77. Should be 1 for proper compression.
    pub(crate) use_lz77: u32,
    /// must be a power of two <= 32768. higher compresses more but is slower. Typical value: 2048.
    pub(crate) windowsize: u32,
    /// mininum lz77 length. 3 is normally best, 6 can be better for some PNGs. Default: 0
    pub(crate) minmatch: u32,
    /// stop searching if >= this length found. Set to 258 for best compression. Default: 128
    pub(crate) nicematch: u32,
    /// use lazy matching: better compression but a bit slower. Default: true
    pub(crate) lazymatching: u32,
}

/// The information of a `Time` chunk in PNG
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Time {
    pub(crate) year: u32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
    pub(crate) minute: u32,
    pub(crate) second: u32,
}

/// Information about the PNG image, except pixels, width and height
#[repr(C)]
#[derive(Debug)]
pub(crate) struct Info {
    /// compression method of the original file. Always 0.
    pub(crate) compression_method: u32,
    /// filter method of the original file
    pub(crate) filter_method: u32,
    /// interlace method of the original file
    pub(crate) interlace_method: u32,
    /// color type and bits, palette and transparency of the PNG file
    pub(crate) color: ColorMode,

    ///  suggested background color chunk (bKGD)
    ///  This color uses the same color mode as the PNG (except alpha channel), which can be 1-bit to 16-bit.
    ///
    ///  For greyscale PNGs, r, g and b will all 3 be set to the same. When encoding
    ///  the encoder writes the red one. For palette PNGs: When decoding, the RGB value
    ///  will be stored, not a palette index. But when encoding, specify the index of
    ///  the palette in background_r, the other two are then ignored.
    ///
    ///  The decoder does not use this background color to edit the color of pixels.
    pub(crate) background_defined: u32,
    /// red component of suggested background color
    pub(crate) background_r: u32,
    /// green component of suggested background color
    pub(crate) background_g: u32,
    /// blue component of suggested background color
    pub(crate) background_b: u32,

    /// Text chunks.
    pub(crate) text: Vec<TextChunk>,
    /// IText Chunks
    pub(crate) itext: Vec<ITextChunk>,

    /// set to 1 to make the encoder generate a tIME chunk
    pub(crate) time_defined: u32,
    /// time chunk (tIME)
    pub(crate) time: Time,

    /// if 0, there is no pHYs chunk and the values below are undefined, if 1 else there is one
    pub(crate) phys_defined: u32,
    /// pixels per unit in x direction
    pub(crate) phys_x: u32,
    /// pixels per unit in y direction
    pub(crate) phys_y: u32,
    /// may be 0 (unknown unit) or 1 (metre)
    pub(crate) phys_unit: u32,

    /// unknown chunks
    /// There are 3 buffers, one for each position in the PNG where unknown chunks can appear
    /// each buffer contains all unknown chunks for that position consecutively
    /// The 3 buffers are the unknown chunks between certain critical chunks:
    /// 0: IHDR-`PLTE`, 1: `PLTE`-IDAT, 2: IDAT-IEND
    /// Do not allocate or traverse this data yourself. Use the chunk traversing functions declared
    /// later, such as lodepng_chunk_next and lodepng_chunk_append, to read/write this struct.
    pub(crate) unknown_chunks_data: [Vec<u8>; 3],
}

/// Settings for the decoder. This contains settings for the PNG and the Zlib decoder, but not the `Info` settings from the `Info` structs.
#[derive(Clone, Debug)]
pub(crate) struct DecoderSettings {
    /// in here is the setting to ignore Adler32 checksums
    pub(crate) zlibsettings: DecompressSettings,
    /// check CRC checksums
    pub(crate) check_crc: bool,
    pub(crate) color_convert: u32,
}

/// automatically use color type with less bits per pixel if losslessly possible. Default: `AUTO`
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FilterStrategy {
    /// every filter at zero
    Zero = 0,
    /// Use filter that gives minumum sum, as described in the official PNG filter heuristic.
    Minsum,
    /// Use the filter type that gives smallest Shannon entropy for this scanline. Depending
    /// on the image, this is better or worse than minsum.
    Entropy,
    /// Brute-force-search PNG filters by compressing each filter for each scanline.
    /// Experimental, very slow, and only rarely gives better compression than MINSUM.
    BruteForce,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub(crate) struct EncoderSettings {
    /// settings for the zlib encoder, such as window size, ...
    pub(crate) zlibsettings: CompressSettings,
    /// how to automatically choose output PNG color type, if at all
    pub(crate) auto_convert: u32,
    /// If true, follows the official PNG heuristic: if the PNG uses a palette or lower than
    /// 8 bit depth, set all filters to zero. Otherwise use the filter_strategy. Note that to
    /// completely follow the official PNG heuristic, filter_palette_zero must be true and
    /// filter_strategy must be FilterStrategy::MINSUM
    pub(crate) filter_palette_zero: u32,
    /// Which filter strategy to use when not using zeroes due to filter_palette_zero.
    /// Set filter_palette_zero to 0 to ensure always using your chosen strategy. Default: FilterStrategy::MINSUM
    pub(crate) filter_strategy: FilterStrategy,

    /// force creating a `PLTE` chunk if colortype is 2 or 6 (= a suggested palette).
    /// If colortype is 3, `PLTE` is _always_ created
    pub(crate) force_palette: u32,
    /// add LodePNG identifier and version as a text chunk, for debugging
    pub(crate) add_id: u32,
    /// encode text chunks as zTXt chunks instead of tEXt chunks, and use compression in iTXt chunks
    pub(crate) text_compression: u32,
}

/// The settings, state and information for extended encoding and decoding
#[repr(C)]
#[derive(Clone, Debug)]
pub(crate) struct State {
    pub(crate) decoder: DecoderSettings,
    pub(crate) encoder: EncoderSettings,

    /// specifies the format in which you would like to get the raw pixel buffer
    pub(crate) info_raw: ColorMode,
    /// info of the PNG image obtained after decoding
    pub(crate) info_png: Info,
    pub(crate) error: super::Error,
}

/// Gives characteristics about the colors of the image, which helps decide which color model to use for encoding.
/// Used internally by default if `auto_convert` is enabled. Public because it's useful for custom algorithms.
#[repr(C)]
pub(crate) struct ColorProfile {
    /// not greyscale
    pub(crate) colored: u32,
    /// image is not opaque - use color key instead of full alpha
    ///
    /// key values, always as 16-bit, in 8-bit case the byte is duplicated, e.g.
    /// 65535 means 255
    pub(crate) key: Option<(u16, u16, u16)>,
    /// image is not opaque and alpha channel or alpha palette required
    pub(crate) alpha: bool,
    /// amount of colors, up to 257. Not valid if bits == 16.
    pub(crate) numcolors: u32,
    /// Remembers up to the first 256 RGBA colors, in no particular order
    pub(crate) palette: [pix::SRgba8; 256],
    /// bits per channel (not for palette). 1,2 or 4 for greyscale only. 16 if 16-bit per channel required.
    pub(crate) bits: u32,
}

impl fmt::Debug for ColorProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ColorProfile")
    }
}

impl fmt::Debug for CompressSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CompressSettings")
    }
}
