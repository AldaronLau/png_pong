use std::io::{ErrorKind, Read};

use crate::{
    consts,
    decode::{Chunks, Error, Result, Steps},
    Step,
};

/// Chunk parser.
#[derive(Debug)]
pub(crate) struct Parser<R: Read> {
    /// Chunk length
    length: u32,
    /// CRC32
    chksum: u32,
    /// Decoder
    decode: Decoder<R>,
    /// Palette chunk found?
    palette: bool,
}

impl<R: Read> Parser<R> {
    /// Prepare a chunk for reading, returning it's name.
    pub(crate) fn prepare(&mut self) -> Result<Option<[u8; 4]>> {
        let first = match self.u8() {
            Ok(first) => first,
            Err(Error::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => {
                return Ok(None)
            }
            Err(e) => return Err(e),
        };
        self.length =
            u32::from_be_bytes([first, self.u8()?, self.u8()?, self.u8()?]);
        // Start checksum over
        self.chksum = consts::CRC32_INIT;
        // Return chunk name
        let name = [self.u8()?, self.u8()?, self.u8()?, self.u8()?];
        if self.length > consts::MAX_CHUNK_SIZE as u32 {
            return Err(Error::ChunkLength(name));
        }
        Ok(Some(name))
    }

    /// Call this when palette chunk is found, whether or not it shows up
    /// influences how other chunks are parsed.
    pub(crate) fn set_palette(&mut self) {
        self.palette = true;
    }

    /// Has palette been parsed yet?
    pub(crate) fn has_palette(&self) -> bool {
        self.palette
    }

    /// Get the length of the chunk.
    pub(crate) fn len(&self) -> usize {
        self.length.try_into().unwrap()
    }

    /// Read and ignore the entire chunk.
    pub(crate) fn unknown_chunk(&mut self) -> Result<Vec<u8>> {
        self.vec(self.len())
    }

    /// Read into a `Vec<u8>`.
    pub(crate) fn vec(&mut self, len: usize) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(self.u8()?);
        }
        Ok(out)
    }

    /// Read entire chunk into a `Vec<u8>`.
    pub(crate) fn raw(&mut self) -> Result<Vec<u8>> {
        self.vec(self.len())
    }

    /// Get a u8 out of the reader.
    pub(crate) fn u8(&mut self) -> Result<u8> {
        let mut byte = [0; 1];
        self.decode
            .reader
            .read_exact(&mut byte)
            .map_err(Error::from)?;
        let index: usize = (self.chksum as u8 ^ byte[0]).into();
        self.chksum = consts::CRC32_LOOKUP[index] ^ (self.chksum >> 8);
        Ok(byte[0])
    }

    /// Get a u16 out of a reader
    pub(crate) fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes([self.u8()?, self.u8()?]))
    }

    /// Get a u32 out of a reader
    pub(crate) fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes([
            self.u8()?,
            self.u8()?,
            self.u8()?,
            self.u8()?,
        ]))
    }

    /// Get a Null terminated String out of a reader
    pub(crate) fn str(&mut self) -> Result<String> {
        let mut bytes = [0u8; 4];
        let mut index = 0;
        let mut out = String::new();
        loop {
            let byte = self.u8()?;
            if byte == 0 {
                break;
            }
            bytes[index] = byte;
            index += 1;
            match std::str::from_utf8(&bytes[0..index]) {
                Ok(c) => {
                    out.push_str(c);
                    index = 0;
                }
                Err(e) => {
                    if e.error_len().is_some() {
                        out.push(std::char::REPLACEMENT_CHARACTER);
                        index = 0;
                    }
                }
            }
        }
        Ok(out)
    }

    /// Get a String out of a reader
    pub(crate) fn string(&mut self, length: usize) -> Result<String> {
        let mut bytes = [0u8; 4];
        let mut index = 0;
        let mut out = String::new();
        for _ in 0..length {
            let byte = self.u8()?;
            bytes[index] = byte;
            index += 1;
            match std::str::from_utf8(&bytes[0..index]) {
                Ok(c) => {
                    out.push_str(c);
                    index = 0;
                }
                Err(e) => {
                    if e.error_len().is_some() {
                        out.push(std::char::REPLACEMENT_CHARACTER);
                        index = 0;
                    }
                }
            }
        }
        Ok(out)
    }

    /// Check if the CRC matches calculated CRC.
    pub(crate) fn check_crc(&mut self, name: &[u8; 4]) -> Result<()> {
        let mut crc32 = [0; 4];
        self.decode.reader.read_exact(&mut crc32)?;
        if u32::from_be_bytes(crc32) != (self.chksum ^ consts::CRC32_INIT) {
            return Err(Error::Crc32(*name));
        }
        Ok(())
    }
}

/// PNG file decoder
///
/// Can be converted into one of two iterators:
/// - [into_iter] / [into_steps] for high-level [Step]s
/// - [into_chunks] for low-level [Chunk]s
///
/// [into_iter]: struct.Decoder.html#method.into_iter
/// [into_steps]: struct.Decoder.html#method.into_steps
/// [into_chunks]: struct.Decoder.html#method.into_chunks
/// [Step]: struct.Step.html
/// [Chunk]: chunk/enum.Chunk.html
#[derive(Debug)]
pub struct Decoder<R: Read> {
    // The source of PNG input.
    reader: R,
}

impl<R: Read> Decoder<R> {
    /// Create a new PNG decoder.  Returns `Err` if it's not a PNG file.
    pub fn new(mut reader: R) -> Result<Self> {
        // Read first 8 bytes (PNG Signature)
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf).map_err(Error::from)?;
        if buf != crate::consts::PNG_SIGNATURE {
            return Err(Error::InvalidSignature);
        }

        Ok(Decoder { reader })
    }

    /// Convert into a `Chunk` iterator.
    pub fn into_chunks(self) -> Chunks<R> {
        Chunks::new(self.parser())
    }

    /// Convert into a `Step` iterator.
    pub fn into_steps(self) -> Steps<R> {
        Steps::new(self.into_chunks())
    }

    /// Convert into a `Parser`.
    fn parser(self) -> Parser<R> {
        Parser {
            decode: self,
            length: 0,
            chksum: 0,
            palette: false,
        }
    }
}

impl<R: Read> IntoIterator for Decoder<R> {
    type IntoIter = Steps<R>;
    type Item = Result<Step>;

    /// Convert into a raster step `Iterator`
    fn into_iter(self) -> Self::IntoIter {
        self.into_steps()
    }
}
