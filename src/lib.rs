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
//! let data = std::fs::read("graphic.png").expect("Failed to open PNG");
//! let data = std::io::Cursor::new(data);
//! let decoder = png_pong::FrameDecoder::<_, pix::SRgba8>::new(data);
//! let png_pong::Frame { raster, delay } = decoder
//!     .last()
//!     .expect("No frames in PNG")
//!     .expect("PNG parsing error");
//! ```
//!
//! - Say you want to save a raster as a PNG file.
//! ```rust,no_run
//! let raster = pix::RasterBuilder::new().with_pixels(1, 1, &[
//!     pix::SRgba8::new(0, 0, 0, 0)][..]
//! );
//! let mut out_data = Vec::new();
//! let mut encoder = png_pong::FrameEncoder::<_, pix::SRgba8>::new(
//!     &mut out_data
//! );
//! let frame = png_pong::Frame{ raster, delay: 0 };
//! encoder.encode(&frame).expect("Failed to add frame");
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

pub use crate::lodepng::Error as ParseError;

// Modules
mod format;
mod frame;
mod error;
mod chunk_encoder;
mod chunk_decoder;
mod frame_encoder;
mod frame_decoder;

pub use format::Format;
pub use frame::Frame;
pub use error::{EncodeError, DecodeError};
pub use chunk_encoder::ChunkEncoder;
pub use chunk_decoder::ChunkDecoder;
pub use frame_encoder::FrameEncoder;
pub use frame_decoder::FrameDecoder;
