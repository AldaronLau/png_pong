// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::{chunk::Chunk, encode::Error};
use std::io::Write;

/// Chunk Encoder for PNG files.
///
/// Note that this doesn't enforce correct ordering of chunks or valid chunk
/// combinations.  If you need it, use `StepEncoder`, the higher-level API.
#[derive(Default, Debug)]
pub struct ChunkEncoder<W: Write> {
    // FIXME: use .encode() instead of pub(crate).
    pub(crate) bytes: W,
    // Compression level
    pub(crate) level: u8,
}

impl<W: Write> ChunkEncoder<W> {
    /// Create a new encoder.  Compression level (0 thru 10)
    pub fn new(w: W, level: u8) -> Self {
        ChunkEncoder { bytes: w, level }
    }

    /// Encode one [`Chunk`](struct.Chunk.html)
    pub fn encode(&mut self, chunk: &mut Chunk) -> Result<(), Error> {
        use Chunk::*;
        match chunk {
            ImageHeader(image_header) => image_header.write(&mut self.bytes),
            ImageData(image_data) => {
                image_data.write(&mut self.bytes, self.level)
            }
            ImageEnd(image_end) => image_end.write(&mut self.bytes),
            Palette(palette) => palette.write(&mut self.bytes),
            Background(background) => background.write(&mut self.bytes),
            InternationalText(itext) => {
                itext.write(&mut self.bytes, self.level)
            }
            Physical(physical) => physical.write(&mut self.bytes),
            Text(text) => text.write(&mut self.bytes),
            Time(time) => time.write(&mut self.bytes),
            Transparency(transparency) => transparency.write(&mut self.bytes),
            CompressedText(ztext) => ztext.write(&mut self.bytes, self.level),
        }
    }
}
