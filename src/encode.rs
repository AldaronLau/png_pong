// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! PNG file encoding

mod chunk_enc;
mod error;
pub(super) mod filter;
mod step_enc; // Share with unfilter

pub use chunk_enc::ChunkEnc;
pub use error::{Error, Result};
pub use filter::FilterStrategy;
pub use step_enc::StepEnc;
