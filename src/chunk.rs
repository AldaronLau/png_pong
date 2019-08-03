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
pub enum Chunk {
    /// Non-International text chunk.
    Text(TextChunk),
    /// International text chunk.
    IText(ITextChunk),
}
