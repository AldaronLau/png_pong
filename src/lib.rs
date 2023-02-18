//! A library for decoding and encoding PNG images and APNG animations.
//!
//! ## Getting Started
//! Add the following to your `Cargo.toml`.
//!
//! ```toml
//! [dependencies.png_pong]
//! version = "0.7"
//! ```
//!
//! ### Example
//! ```rust
//! // Saving raster as a PNG file
//! let raster = png_pong::PngRaster::Rgba8(pix::Raster::with_pixels(1, 1, &[
//!     pix::rgb::SRgba8::new(0, 0, 0, 0)][..]
//! ));
//! let mut out_data = Vec::new();
//! let mut encoder = png_pong::Encoder::new(&mut out_data).into_step_enc();
//! let step = png_pong::Step{ raster, delay: 0 };
//! encoder.encode(&step).expect("Failed to add frame");
//! std::fs::write("graphic.png", out_data).expect("Failed to save image");
//!
//! // Loading PNG file into a Raster
//! let data = std::fs::read("graphic.png").expect("Failed to open PNG");
//! let data = std::io::Cursor::new(data);
//! let decoder = png_pong::Decoder::new(data).expect("Not PNG").into_steps();
//! let png_pong::Step { raster, delay } = decoder
//!     .last()
//!     .expect("No frames in PNG")
//!     .expect("PNG parsing error");
//! ```

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/AldaronLau/png_pong/master/res/icon.png",
    html_favicon_url = "https://raw.githubusercontent.com/AldaronLau/png_pong/master/res/icon.png",
    html_root_url = "https://docs.rs/png_pong"
)]
#![forbid(unsafe_code)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

pub mod chunk;
pub mod decode;
pub mod encode;

pub(crate) mod decoder;

mod adam7;
mod bitstream;
mod consts;
mod encoder;
mod raster;
mod step;
mod zlib;

pub use decoder::Decoder;
pub use encoder::Encoder;
pub use raster::PngRaster;
pub use step::Step;
