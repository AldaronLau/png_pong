/// PNG Pong Encoder Result Type
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(std::sync::Arc::new(err))
    }
}

/// Encoding Errors.
#[derive(Debug)]
#[allow(variant_size_differences)]
pub enum Error {
    /// A wrapped I/O error.
    Io(std::sync::Arc<std::io::Error>),
    /// Chunks arranged in invalid sequence. (FIXME: Replace with ChunkOrder)
    InvalidChunkSequence,
    /// Chunk is too large to save in a PNG file (length must fit in 32 bits)
    ChunkTooBig,
    /// key is not between 1-79 characters
    KeySize(usize),
    /// PLTE chunk with a palette that has less than 1 or more than 256 colors
    BadPalette,
    /// Chunks arranged in invalid sequence.  Provides PNG chunk identifier of
    /// the out-of-order chunk.
    ChunkOrder([u8; 4]),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Io(io) => write!(f, "I/O Error: {}", io),
            InvalidChunkSequence => write!(f, "Invalid chunk sequence"),
            ChunkTooBig => write!(f, "Chunk too big"),
            KeySize(size) => {
                write!(f, "Key size {size} is not between 1 and 79 characters")
            }
            BadPalette => write!(f, "Invalid palette"),
            ChunkOrder(bytes) => write!(
                f,
                "Chunk {} out of order",
                String::from_utf8_lossy(bytes)
            ),
        }
    }
}

impl std::error::Error for Error {}
