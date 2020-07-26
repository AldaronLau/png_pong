// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/// PNG file encoder
///
/// FIXME
/// Can be converted into one of two iterators:
/// - [into_iter] / [into_steps] for high-level [Step]s
/// - [into_blocks] for low-level [Chunk]s
///
/// [into_iter]: struct.Decoder.html#method.into_iter
/// [into_steps]: struct.Decoder.html#method.into_steps
/// [into_chunks]: struct.Decoder.html#method.into_chunks
/// [Step]: struct.Step.html
/// [Chunk]: struct.Chunk.html
pub struct Encoder {}
