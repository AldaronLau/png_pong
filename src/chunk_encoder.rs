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
