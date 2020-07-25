// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! PNG file encoding

mod chunk_encoder;
mod error;
mod step_encoder;

pub use chunk_encoder::ChunkEncoder;
pub use error::{Error, Result};
pub use step_encoder::StepEncoder;
