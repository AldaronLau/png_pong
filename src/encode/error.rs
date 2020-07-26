// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/// PNG Pong Encoder Result Type
pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(std::sync::Arc::new(err))
    }
}

/// Encoding Errors.
#[derive(Debug)]
#[allow(variant_size_differences)]
pub enum Error {
    /// A wrapped I/O error.
    Io(std::sync::Arc<std::io::Error>),
    /// Chunks arranged in invalid sequence. (FIXME: Replace with ChunkOrder)
    InvalidChunkSequence,
    /// Chunk is too large to save in a PNG file (length must fit in 32 bits)
    ChunkTooBig,
    /// Text is not between 1-79 characters
    TextSize(usize),
    /// PLTE chunk with a palette that has less than 1 or more than 256 colors
    BadPalette,
    /// Chunks arranged in invalid sequence.  Provides PNG chunk identifier of
    /// the out-of-order chunk.
    ChunkOrder([u8; 4]),
}
