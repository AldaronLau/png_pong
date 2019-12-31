//! # PNG Pong - A pure Rust PNG encoder & decoder
//! This is a pure Rust PNG image decoder and encoder based on lodepng.  This crate allows easy reading and writing of PNG files without any system dependencies.
//!
//! ## Goals
//! - Forbid unsafe.
//! - APNG support as iterator.
//! - Fast.
//! - Compatible with pix / gift-style API.
//! - Load all PNG files crushed with pngcrush.
//! - Save crushed PNG files.
//! - Clean, well-documented, concise code.
//!
//! ## Examples
//! - Say you want to read a PNG file into a raster:
//! ```rust,no_run
//! let mut decoder_builder = png_pong::DecoderBuilder::new();
//! let data = std::fs::read("graphic.png").expect("Failed to open PNG");
//! let data = std::io::Cursor::new(data);
//! let decoder = decoder_builder.decode_rasters(data);
//! let (raster, _nanos) = decoder
//!     .last()
//!     .expect("No frames in PNG")
//!     .expect("PNG parsing error");
//! ```
//!
//! - Say you want to save a raster as a PNG file.
//! ```rust,no_run
//! let raster = png_pong::RasterBuilder::new().with_pixels(1, 1, &[
//!     pix::Rgba8::with_alpha(
//!         pix::Ch8::new(0),
//!         pix::Ch8::new(0),
//!         pix::Ch8::new(0),
//!         pix::Ch8::new(0),
//!     )][..]
//! );
//! let mut out_data = Vec::new();
//! let mut encoder = png_pong::EncoderBuilder::new();
//! let mut encoder = encoder.encode_rasters(&mut out_data);
//! encoder.add_frame(&raster, 0).expect("Failed to add frame");
//! std::fs::write("graphic.png", out_data).expect("Failed to save image");
//! ```
//!
//! ## TODO
//! - Implement APNG reading.
//! - Implement Chunk reading (with all the different chunk structs).
//! - RasterDecoder should wrap ChunkDecoder & RasterEncoder should wrap ChunkEncoder
//! - Replace `ParseError` with Rust-style enum instead of having a C integer.
//! - More test cases to test against.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc(
    html_logo_url = "https://plopgrizzly.com/icon.svg",
    html_favicon_url = "https://plopgrizzly.com/icon.svg"
)]

mod lodepng;

/// Low-level chunk control.
///
pub mod chunk;

/// Prelude.
pub mod prelude;

pub use crate::lodepng::Error as ParseError;
pub use pix::{Raster, RasterBuilder};

/// Decoding Errors.
#[derive(Debug)]
pub enum DecodeError {
    /// Couldn't parse file.
    ParseError(ParseError),
    /// Couldn't convert to requested format.
    ConversionError,
    /// Failed to read data.
    ReadError,
}

/// Encoding Errors.
#[derive(Debug)]
pub enum EncodeError {
    /// Couldn't parse file.
    ParseError(ParseError),
    /// Failed to read data.
    WriteError,
}

/// Metadata for raster.
pub struct Meta {}

/// Decoder for iterating [`Chunk`](chunk/enum.Chunk.html)s within a PNG file.
///
/// build with
/// `Decoder.`[`into_chunk_decoder()`](struct.Decoder.html#method.into_chunk_decoder).
#[allow(unused)] // TODO
pub struct ChunkDecoder<'a, R>
where
    R: std::io::Read,
{
    state: &'a mut lodepng::State,
    bytes: R,
}

/// Encoder for writing [`Chunk`](chunk/enum.Chunk.html)s into a PNG file.
#[allow(unused)] // TODO
pub struct ChunkEncoder<'a, W>
where
    W: std::io::Write,
{
    state: &'a mut lodepng::State,
    bytes: W,
}

/// Decoder for iterating [`Raster`](chunk/enum.Raster.html)s within a PNG file.
///
/// build with
/// `Decoder.`[`into_raster_decoder()`](struct.Decoder.html#method.into_raster_decoder).
pub struct RasterDecoder<'a, R>
where
    R: std::io::Read,
{
    state: &'a mut lodepng::State,
    bytes: R,
    has_decoded: bool,
}

impl<'a, R> RasterDecoder<'a, R>
where
    R: std::io::Read,
{
    /// Get metadata from Raster.
    pub fn meta(&self) -> Meta {
        Meta {}
    }
}

impl<'a, R> Iterator for RasterDecoder<'a, R>
where
    R: std::io::Read,
{
    /// First element in tuple is the Raster.  The second is nanoseconds between
    /// this frame and the next (0 for stills).
    type Item = Result<(pix::Raster<pix::Rgba8>, u64), DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_decoded {
            return None;
        }

        self.has_decoded = true;

        let mut bytes: Vec<u8> = vec![];

        let raster = if self.bytes.read_to_end(&mut bytes).is_ok() {
            match self.state.decode(bytes) {
                Ok(o) => match o {
                    lodepng::Image::RGBA(img) => Ok((img, 0)),
                    _ => Err(DecodeError::ConversionError),
                },
                Err(e) => Err(DecodeError::ParseError(e)),
            }
        } else {
            Err(DecodeError::ReadError)
        };

        Some(raster)
    }
}

use std::marker::PhantomData;

/// Encoder for writing [`Raster`](chunk/enum.Raster.html)s into a PNG file.
pub struct RasterEncoder<'a, W, F>
where
    W: std::io::Write,
    F: pix::Format,
{
    state: &'a mut lodepng::State,
    bytes: W,
    _phantom: PhantomData<F>,
}

impl<'a, W, F> RasterEncoder<'a, W, F>
where
    W: std::io::Write,
    F: pix::Format,
{
    /// Add a frame to the animation or still.
    pub fn add_frame(
        &mut self,
        raster: &pix::Raster<F>,
        nanos: u64,
    ) -> Result<(), EncodeError> {
        let _ = nanos; // TODO

        let bytes = match self.state.encode(raster) {
            Ok(o) => o,
            Err(e) => return Err(EncodeError::ParseError(e)),
        };

        match self.bytes.write(&bytes) {
            Ok(_size) => Ok(()),
            Err(_) => return Err(EncodeError::WriteError),
        }
    }
}

/// Builder for PNG decoders.
/// - [`ChunkDecoder`](struct.ChunkDecoder.html) - low-level, [`Chunk`](chunk/enum.Chunk.html)s
/// - [`RasterDecoder`](struct.RasterDecoder.html) - high-level, [`Raster`](struct.Raster.html)s
#[derive(Default)]
pub struct DecoderBuilder {
    state: lodepng::State,
}

impl DecoderBuilder {
    /// Create a new Decoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check CRC checksum.  CRC checksums are ignored by default for speed.
    pub fn check_crc(mut self) -> Self {
        self.state.decoder.check_crc = true;
        self
    }

    /// Check Adler32 checksum.  Adler32 checksums are ignored by default for
    /// speed.
    pub fn check_adler32(mut self) -> Self {
        self.state.decoder.zlibsettings.check_adler32 = true;
        self
    }

    /// Convert into a chunk decoder.
    pub fn decode_chunks<'a, R>(&'a mut self, bytes: R) -> ChunkDecoder<'a, R>
    where
        R: std::io::Read,
    {
        ChunkDecoder {
            state: &mut self.state,
            bytes,
        }
    }

    /// Convert into a raster decoder.
    pub fn decode_rasters<'a, R>(&'a mut self, bytes: R) -> RasterDecoder<'a, R>
    where
        R: std::io::Read,
    {
        RasterDecoder {
            state: &mut self.state,
            bytes,
            has_decoded: false,
        }
    }
}

/// Builder for PNG encoders.
#[derive(Default)]
pub struct EncoderBuilder {
    state: lodepng::State,
}

impl EncoderBuilder {
    /// Create a new encoder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert into a chunk encoder.
    pub fn encode_chunks<'a, W>(&'a mut self, bytes: W) -> ChunkEncoder<'a, W>
    where
        W: std::io::Write,
    {
        ChunkEncoder {
            state: &mut self.state,
            bytes,
        }
    }

    /// Convert into a raster encoder.
    pub fn encode_rasters<'a, W, F>(&'a mut self, bytes: W) -> RasterEncoder<'a, W, F>
    where
        W: std::io::Write,
        F: pix::Format,
    {
        RasterEncoder {
            state: &mut self.state,
            bytes,
            _phantom: PhantomData,
        }
    }
}
