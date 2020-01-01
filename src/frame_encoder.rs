use std::{io::{self, Write}, marker::PhantomData};
use pix::Raster;
use crate::{Frame, ChunkEncoder, Format};

/// Frame Encoder for PNG files.
pub struct FrameEncoder<W: Write, F: Format> {
    encoder: ChunkEncoder<W>,
    _phantom: PhantomData<F>,
}

impl<W: Write, F: Format> FrameEncoder<W, F> {
    /// Create a new encoder.
    pub fn new(w: W) -> Self {
        FrameEncoder {
            encoder: ChunkEncoder::new(w),
            _phantom: PhantomData,
        }
    }

    /// Encode a still.
    pub fn still(&mut self, raster: &Raster<F>) -> io::Result<()> {
        self.encoder.state.info_raw.colortype = F::PNG_COLOR;
        self.encoder.state.info_raw.set_bitdepth(F::BIT_DEPTH);
        self.encoder.state.info_png.color.colortype = F::PNG_COLOR;
        self.encoder.state.info_png.color.set_bitdepth(F::BIT_DEPTH);

        let bytes = match self.encoder.state.encode(&raster) {
            Ok(o) => o,
            Err(e) => panic!("Encoding failure bug: {}!", e),
        };

        match self.encoder.bytes.write(&bytes) {
            Ok(_size) => Ok(()),
            Err(e) => return Err(e),
        }
    }

    /// Encode one [`Frame`](struct.Frame.html)
    pub fn encode(&mut self, frame: &Frame<F>) -> io::Result<()> {
        self.encoder.state.info_raw.colortype = F::PNG_COLOR;
        self.encoder.state.info_raw.set_bitdepth(F::BIT_DEPTH);
        self.encoder.state.info_png.color.colortype = F::PNG_COLOR;
        self.encoder.state.info_png.color.set_bitdepth(F::BIT_DEPTH);

        let bytes = match self.encoder.state.encode(&frame.raster) {
            Ok(o) => o,
            Err(e) => panic!("Encoding failure bug: {}!", e),
        };

        match self.encoder.bytes.write(&bytes) {
            Ok(_size) => Ok(()),
            Err(e) => return Err(e),
        }
    }
}