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

use std::io::Write;
use crate::{EncodeError, lodepng, chunk::Chunk};

/// Chunk Encoder for PNG files.
#[derive(Default)]
pub struct ChunkEncoder<W: Write> {
    // FIXME: use .encode() instead of pub(crate).
    pub(crate) state: lodepng::State,
    pub(crate) bytes: W,
}

impl<W: Write> ChunkEncoder<W> {
    /// Create a new encoder.
    pub fn new(w: W) -> Self {
        ChunkEncoder {
            state: lodepng::State::default(),
            bytes: w,
        }
    }

    /// Encode one [`Chunk`](struct.Chunk.html)
    pub fn encode(&mut self, chunk: &mut Chunk) -> Result<(), EncodeError> {
        // FIXME
        let _chunk = chunk;
        Ok(())
    }
}
