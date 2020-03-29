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

use crate::ParseError;

/// Decoding Errors.
#[derive(Debug, Copy, Clone)]
pub enum DecodeError {
    /// A wrapped I/O error.
    Io(std::io::ErrorKind),
    /// Couldn't parse file.
    ParseError(ParseError),
    /// PNG file has different color `Format` than `Raster`.
    Color,
    /// PNG file has different bit depth than `Raster`.
    BitDepth,
}

/// Decoding Errors.
#[derive(Debug, Copy, Clone)]
pub enum EncodeError {
    /// A wrapped I/O error.
    Io(std::io::ErrorKind),
    /// Chunks arranged in invalid sequence.
    InvalidChunkSequence,
}
