use crate::ParseError;

/// Decoding Errors.
#[derive(Debug)]
pub enum DecodeError {
    /// A wrapped I/O error.
    Io(std::io::Error),
    /// Couldn't parse file.
    ParseError(ParseError),
    /// PNG file has different color `Format` than `Raster`.
    Color,
    /// PNG file has different bit depth than `Raster`.
    BitDepth,
}

/// Decoding Errors.
#[derive(Debug)]
pub enum EncodeError {
    /// A wrapped I/O error.
    Io(std::io::Error),
    /// Chunks arranged in invalid sequence.
    InvalidChunkSequence,
}
