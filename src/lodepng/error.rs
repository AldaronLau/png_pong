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

use std::error;
use std::fmt;
use std::io;

/// A lame error code.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Error(pub u8);

impl Error {
    /// Returns an English description of the numerical error code.
    pub fn as_str(self) -> &'static str {
        self.description_en()
    }

    /// Helper function for the library
    pub(crate) fn to_result(self) -> Result<(), Error> {
        match self {
            Error(0) => Ok(()),
            err => Err(err),
        }
    }
}

impl From<Error> for Result<(), Error> {
    fn from(err: Error) -> Self {
        err.to_result()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.as_str(), self.0)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        match err.kind() {
            io::ErrorKind::NotFound | io::ErrorKind::UnexpectedEof => Error(78),
            _ => Error(79),
        }
    }
}

/*
This returns the description of a numerical error code in English. This is also
the documentation of all the error codes.
*/
impl Error {
    fn description_en(self) -> &'static str {
        match self.0 {
            0 => "no error, everything went ok",
            1 => "nothing done yet",

            /*the Encoder/Decoder has done nothing yet, error checking makes no sense yet*/
            10 => "end of input memory reached without huffman end code",

            /*while huffman decoding*/
            11 => "error in code tree made it jump outside of huffman tree",

            /*while huffman decoding*/
            13 | 14 | 15 => "problem while processing dynamic deflate block",
            16 => "unexisting code while processing dynamic deflate block",
            18 => "invalid distance code while inflating",
            17 | 19 | 22 => "end of out buffer memory reached while inflating",
            20 => "invalid deflate block BTYPE encountered while decoding",
            21 => "NLEN is not ones complement of LEN in a deflate block",

            /*end of out buffer memory reached while inflating:
                This can happen if the inflated deflate data is longer than the amount of bytes required to fill up
                all the pixels of the image, given the color depth and image dimensions. Something that doesn't
                happen in a normal, well encoded, PNG image.*/
            23 => "end of in buffer memory reached while inflating",
            24 => "invalid FCHECK in zlib header",
            25 => "invalid compression method in zlib header",
            26 => "FDICT encountered in zlib header while it\'s not used for PNG",
            27 => "PNG file is smaller than a PNG header",
            /*Checks the magic file header, the first 8 bytes of the PNG file*/
            28 => "incorrect PNG signature, it\'s no PNG or corrupted",
            29 => "first chunk is not the header chunk",
            30 => "chunk length too large, chunk broken off at end of file",
            31 => "illegal PNG color type or bpp",
            32 => "illegal PNG compression method",
            33 => "illegal PNG filter method",
            34 => "illegal PNG interlace method",
            35 => "chunk length of a chunk is too large or the chunk too small",
            36 => "illegal PNG filter type encountered",
            37 => "illegal bit depth for this color type given",
            38 => "the palette is too big",
            /*more than 256 colors*/
            39 => "more palette alpha values given in tRNS chunk than there are colors in the palette",
            40 => "tRNS chunk has wrong size for greyscale image",
            41 => "tRNS chunk has wrong size for RGB image",
            42 => "tRNS chunk appeared while it was not allowed for this color type",
            43 => "bKGD chunk has wrong size for palette image",
            44 => "bKGD chunk has wrong size for greyscale image",
            45 => "bKGD chunk has wrong size for RGB image",
            48 => "empty input buffer given to decoder. Maybe caused by non-existing file?",
            49 | 50 => "jumped past memory while generating dynamic huffman tree",
            51 => "jumped past memory while inflating huffman block",
            52 => "jumped past memory while inflating",
            53 => "size of zlib data too small",
            54 => "repeat symbol in tree while there was no value symbol yet",

            /*jumped past tree while generating huffman tree, this could be when the
               tree will have more leaves than symbols after generating it out of the
               given lenghts. They call this an oversubscribed dynamic bit lengths tree in zlib.*/
            55 => "jumped past tree while generating huffman tree",
            56 => "given output image colortype or bitdepth not supported for color conversion",
            57 => "invalid CRC encountered (checking CRC can be disabled)",
            58 => "invalid ADLER32 encountered (checking ADLER32 can be disabled)",
            59 => "requested color conversion not supported",
            60 => "invalid window size given in the settings of the encoder (must be 0-32768)",
            61 => "invalid BTYPE given in the settings of the encoder (only 0, 1 and 2 are allowed)",

            /*LodePNG leaves the choice of RGB to greyscale conversion formula to the user.*/
            62 => "conversion from color to greyscale not supported",
            63 => "length of a chunk too long, max allowed for PNG is 2147483647 bytes per chunk",

            /*(2^31-1)*/
            /*this would result in the inability of a deflated block to ever contain an end code. It must be at least 1.*/
            64 => "the length of the END symbol 256 in the Huffman tree is 0",
            66 => "the length of a text chunk keyword given to the encoder is longer than the maximum of 79 bytes",
            67 => "the length of a text chunk keyword given to the encoder is smaller than the minimum of 1 byte",
            68 => "tried to encode a PLTE chunk with a palette that has less than 1 or more than 256 colors",
            69 => "unknown chunk type with \'critical\' flag encountered by the decoder",
            71 => "unexisting interlace mode given to encoder (must be 0 or 1)",
            72 => "while decoding, unexisting compression method encountering in zTXt or iTXt chunk (it must be 0)",
            73 => "invalid tIME chunk size",
            74 => "invalid pHYs chunk size",
            /*length could be wrong, or data chopped off*/
            75 => "no null termination char found while decoding text chunk",
            76 => "iTXt chunk too short to contain required bytes",
            77 => "integer overflow in buffer size",
            78 => "failed to open file for reading",

            /*file doesn't exist or couldn't be opened for reading*/
            79 => "failed to open file for writing",
            80 => "tried creating a tree of 0 symbols",
            81 => "lazy matching at pos 0 is impossible",
            82 => "color conversion to palette requested while a color isn\'t in palette",
            83 => "memory allocation failed",
            84 => "given image too small to contain all pixels to be encoded",
            86 => "impossible offset in lz77 encoding (internal bug)",
            87 => "must provide custom zlib function pointer if LODEPNG_COMPILE_ZLIB is not defined",
            88 => "invalid filter strategy given for EncoderSettings.filter_strategy",
            89 => "text chunk keyword too short or long: must have size 1-79",

            /*the windowsize in the CompressSettings. Requiring POT(==> & instead of %) makes encoding 12% faster.*/
            90 => "windowsize must be a power of two",
            91 => "invalid decompressed idat size",
            92 => "too many pixels, not supported",
            93 => "zero width or height is invalid",
            94 => "header chunk must have a size of 13 bytes",
            _ => "unknown error code",
        }
    }
}
