use std::io::Read;
use crate::{DecodeError, lodepng, chunk::Chunk};

/// Chunk Decoder for PNG files.
#[derive(Default)]
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
