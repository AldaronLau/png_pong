use std::io::Write;

use crate::{chunk::Chunk, encode::Error, encoder::Enc};

/// Chunk Encoder for PNG files.
///
/// Note that this doesn't enforce correct ordering of chunks or valid chunk
/// combinations.  If you need it, use `StepEncoder`, the higher-level API.
#[derive(Debug)]
pub struct ChunkEnc<W: Write> {
    // FIXME: use .encode() instead of pub(crate).
    pub(crate) enc: Enc<W>,
}

impl<W: Write> ChunkEnc<W> {
    /// Create a new encoder.
    pub(crate) fn new(enc: Enc<W>) -> Self {
        Self { enc }
    }

    /// Encode one [`Chunk`](struct.Chunk.html)
    pub fn encode(&mut self, chunk: &mut Chunk) -> Result<(), Error> {
        use Chunk::*;
        match chunk {
            ImageHeader(image_header) => image_header.write(&mut self.enc),
            ImageData(image_data) => image_data.write(&mut self.enc),
            ImageEnd(image_end) => image_end.write(&mut self.enc),
            Palette(palette) => palette.write(&mut self.enc),
            Background(background) => background.write(&mut self.enc),
            InternationalText(itext) => itext.write(&mut self.enc),
            Physical(physical) => physical.write(&mut self.enc),
            Text(text) => text.write(&mut self.enc),
            Time(time) => time.write(&mut self.enc),
            Transparency(transparency) => transparency.write(&mut self.enc),
            CompressedText(ztext) => ztext.write(&mut self.enc),
            Unknown(unknown) => unknown.write(&mut self.enc),
        }
    }
}
