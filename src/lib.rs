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

#![cfg_attr(feature = "external_doc", feature(external_doc))]
#![cfg_attr(feature = "external_doc", doc(include = "../README.md"))]
#![doc = ""]
#![doc(
    html_logo_url = "https://libcala.github.io/logo.svg",
    html_favicon_url = "https://libcala.github.io/icon.svg",
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

mod lodepng;

/// Low-level chunk control.
///
pub mod chunk;

pub use crate::lodepng::Error as ParseError;
pub use crate::lodepng::ColorType;

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
