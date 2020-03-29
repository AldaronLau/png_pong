// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
// Copyright © 2014-2017 Kornel Lesiński
// Copyright © 2005-2016 Lode Vandevenne
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A PNG file consists of a sequence of [`Chunk`](enum.Chunk.html)s in a
//! specific order.

/// Non-International Text Chunk Data (tEXt and zTXt)
#[derive(Debug)]
pub struct TextChunk {
    /// A keyword that gives a short description of what the text in `val`
    /// represents, e.g. Title, Author, Description, or anything else.  Minimum
    /// of 1 character, and maximum 79 characters long.
    pub key: String,
    /// The actual message.  It's discouraged to use a single line length longer
    /// than 79 characters
    pub val: String,
}

/// International Text Chunk Data (iTXt)
#[derive(Debug)]
pub struct ITextChunk {
    /// A keyword that gives a short description of what the text in `val`
    /// represents, e.g. Title, Author, Description, or anything else.  Minimum
    /// of 1 character, and maximum 79 characters long.
    pub key: String,
    /// Additional string "langtag"
    pub langtag: String,
    /// Additional string "transkey"
    pub transkey: String,
    /// The actual message.  It's discouraged to use a single line length longer
    /// than 79 characters
    pub val: String,
}

/// A chunk within a PNG file.
#[derive(Debug)]
pub enum Chunk {
    /// Non-International text chunk.
    Text(TextChunk),
    /// International text chunk.
    IText(ITextChunk),
}
