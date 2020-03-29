// png-pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
// Copyright © 2014-2017 Kornel Lesiński
// Copyright © 2005-2016 Lode Vandevenne
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::{chunk::Chunk, lodepng, DecodeError};
use std::io::Read;

/// Chunk Decoder for PNG files.
#[derive(Default, Debug)]
pub struct ChunkDecoder<R: Read> {
    // FIXME: use .decode() instead of pub(crate).
    pub(crate) state: lodepng::State,
    pub(crate) bytes: R,
}

impl<R: Read> ChunkDecoder<R> {
    /// Create a new encoder.
    pub fn new(r: R) -> Self {
        ChunkDecoder {
            state: lodepng::State::default(),
            bytes: r,
        }
    }

    /// Decode one [`Chunk`](struct.Chunk.html)
    pub fn decode(&mut self) -> Result<Chunk, DecodeError> {
        // FIXME
        todo!()
    }
}
