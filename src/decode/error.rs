use crate::chunk::ColorType;

/// PNG Pong Decoder Result Type
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(std::sync::Arc::new(err))
    }
}

/// Decoding Errors.
#[derive(Clone, Debug)]
#[allow(variant_size_differences)]
pub enum Error {
    /// A wrapped I/O error.
    Io(std::sync::Arc<std::io::Error>),
    /// Unrecognized color type
    ColorType(u8),
    /// Out of bounds bit depth
    BitDepth(u8),
    /// Invalid color type / bit depth combination
    ColorMode(ColorType, u8),
    /// Pixel size in background color doesn't match pixel size in image data
    BackgroundSize(ColorType),
    /// The first 8 bytes are not the correct PNG signature
    InvalidSignature,
    /// Adler checksum not correct, data must be corrupted
    AdlerChecksum,
    /// Inflate algorithm failure
    Inflate(miniz_oxide::inflate::TINFLStatus),
    /// ZLib compression includes preset dictionary, which is not allowed
    /// according to the PNG specification
    PresetDict,
    /// Invalid compression method in zlib header
    CompressionMethod,
    /// Invalid FCHECK in zlib header
    ZlibHeader,
    /// ZLib data is too small
    ZlibTooSmall,
    /// TODO
    InterlaceMethod,
    /// TODO
    FilterMethod,
    /// TODO
    ImageDimensions,
    /// File doesn't contain any chunks.
    Empty,
    /// Text is not between 1-79 characters
    TextSize(usize),
    /// The length of the END symbol 256 in the Huffman tree is 0
    HuffmanEnd,
    /// Unrecognized filter type
    IllegalFilterType,
    /// Alpha palette is larger than the palette.
    AlphaPaletteLen,
    /// Chunk is the wrong size
    ChunkSize,
    /// Mode has an alpha channel, but also an alpha palette (must pick one)
    AlphaPaletteWithAlphaMode,
    /// Chunk was expected to end, but didn't
    NoEnd,
    /// Invalid unit type
    PhysUnits,
    /// Null terminator is missing.
    NulTerm,
    /// Invalid chunk length for the chunk type
    ChunkLength([u8; 4]),
    /// Not a critical error, should be ignored (chunk not recognized).
    UnknownChunkType([u8; 4]),
    /// Input reading appears to end in the middle of a PNG file
    Eof,
    /// Chunks are out of order
    ChunkOrder,
    /// IDAT Chunk not found.
    NoImageData,
    /// Chunk(s) were found after the IEND chunk.
    TrailingChunk,
    /// Multiple of a chunk were found when only one of this type is allowed.
    Multiple([u8; 4]),
    /// CRC32 Checksum failed for a chunk
    Crc32([u8; 4]),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Io(io) => write!(f, "I/O Error: {}", io),
            ColorType(_) => write!(f, "Unrecognized color type"),
            BitDepth(_) => write!(f, "Out of bounds bit depth"),
            ColorMode(_ct, _bd) => write!(f, "Invalid color type / bit depth combination"),
            BackgroundSize(_) => write!(f, "Background color type mismatch with image color type"),
            InvalidSignature => write!(f, "Not a PNG file"),
            AdlerChecksum => write!(f, "Adler checksum not correct, data must be corrupted"),
            Inflate(e) => write!(f, "Inflate: {:?}", e),
            PresetDict => write!(f, "ZLib compression using preset dictionary, PNG doesn't allow"),
            CompressionMethod => write!(f, "Invalid compression method in zlib header"),
            ZlibHeader => write!(f, "Invalid FCHECK in zlib header"),
            ZlibTooSmall => write!(f, "ZLib data is too small"),
            InterlaceMethod => write!(f, "Invalid interlace method"),
            FilterMethod => write!(f, "Invalid filter method"),
            ImageDimensions => write!(f, "Invalid image dimensions, must be greater than 0"),
            Empty => write!(f, "File doesn't contain any chunks."), // FIXME: NoImageData
            TextSize(size) => write!(f, "Text size ({}) doesn't fit inequality 1 ≤ x ≤ 79", size),
            HuffmanEnd => write!(f, "The length of the END symbol 256 in the Huffman tree is 0"),
            IllegalFilterType => write!(f, "Unrecognized filter type"),
            AlphaPaletteLen => write!(f, "Alpha palette is larger than the palette."),
            ChunkSize => write!(f, "Chunk is the wrong size"), // FIXME: Replace with ChunkLength
            AlphaPaletteWithAlphaMode => write!(f, "Mode has an alpha channel, but also an alpha palette (must pick one)"),
            NoEnd => write!(f, "Chunk was expected to end, but didn't"), // FIXME: Replace with ChunkLength
            PhysUnits => write!(f, "Unknown physical units (must be unspecified or meter)"),
            NulTerm => write!(f, "Expected null terminator, but not found"),
            ChunkLength(bytes) => write!(f, "{} chunk wrong length", String::from_utf8_lossy(bytes)),
            UnknownChunkType(bytes) => write!(f, "{} chunk unrecognized", String::from_utf8_lossy(bytes)),
            Eof => write!(f, "Unexpected end of file"),
            ChunkOrder => write!(f, "PNG chunks are out of order"),
            NoImageData => write!(f, "No IDAT chunk exists, invalid PNG file"),
            TrailingChunk => write!(f, "Trailing chunks were found after IEND, which is invalid"),
            Multiple(bytes) => write!(f, "Only one {} chunk allowed, but found multiple", String::from_utf8_lossy(bytes)),
            Crc32(bytes) => write!(f, "CRC32 Checksum failed for {} chunk", String::from_utf8_lossy(bytes)),
        }
    }
}

impl std::error::Error for Error {}
