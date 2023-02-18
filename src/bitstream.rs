//! Read and write from a reversed bit stream

use std::io::{Bytes, Read, Result, Write};

/// A reversed bit stream writer.
pub(super) struct BitstreamWriter<W: Write> {
    /// Pointer within the stream.
    bitpointer: usize,
    /// Writer
    stream: W,
    /// Current byte
    byte: u8,
}

impl<W: Write> BitstreamWriter<W> {
    /// Create a new `BitstreamReader` from a type that implements `Read`.
    #[inline(always)]
    pub(super) fn new(stream: W) -> Self {
        BitstreamWriter {
            bitpointer: 0,
            stream,
            byte: 0,
        }
    }

    pub(super) fn write(&mut self, bit: bool) -> Result<()> {
        /* the current bit in bitstream must be 0 for this to work */
        if bit {
            /* earlier bit of huffman code is in a lesser significant bit of
             * an earlier byte */
            self.byte |= 1 << (7 - self.bitpointer);
        }
        self.bitpointer += 1;
        if self.bitpointer == 8 {
            self.bitpointer = 0;
            self.stream.write_all(&[self.byte])?;
            self.byte = 0;
        }
        Ok(())
    }
}

/// A reversed bit stream reader.
pub(super) struct BitstreamReader<R: Read> {
    /// Pointer within the stream.
    bitpointer: usize,
    /// Reader
    stream: Bytes<R>,
    /// Current byte
    byte: Option<u8>,
}

impl<R: Read> BitstreamReader<R> {
    /// Create a new `BitstreamReader` from a type that implements `Read`.
    #[inline(always)]
    pub(super) fn new(stream: R) -> Result<Self> {
        let mut stream = stream.bytes();
        Ok(BitstreamReader {
            bitpointer: 0,
            byte: match stream.next() {
                Some(Ok(v)) => Ok(Some(v)),
                Some(Err(e)) => Err(e),
                None => Ok(None),
            }?,
            stream,
        })
    }

    /// Create a new `BitstreamReader` from a type that implements `Read`.
    #[inline(always)]
    pub(super) fn with_bitpointer(
        stream: R,
        bitpointer: usize,
    ) -> Result<Self> {
        let mut stream = stream.bytes();
        for _ in 0..bitpointer / 8 {
            stream.next().unwrap().unwrap();
        }
        Ok(BitstreamReader {
            bitpointer: bitpointer % 8,
            byte: match stream.next() {
                Some(Ok(v)) => Ok(Some(v)),
                Some(Err(e)) => Err(e),
                None => Ok(None),
            }?,
            stream,
        })
    }

    /// Read the next bit from the stream.
    #[inline(always)]
    pub(super) fn read(&mut self) -> Result<Option<bool>> {
        let byte = match self.byte {
            Some(b) => b,
            None => return Ok(None),
        };
        let bitpointer = self.bitpointer;

        // Advance bit pointer
        self.bitpointer += 1;
        if self.bitpointer == 8 {
            self.bitpointer = 0;
            self.byte = match self.stream.next() {
                Some(Ok(v)) => Ok(Some(v)),
                Some(Err(e)) => Err(e),
                None => Ok(None),
            }?;
        }

        Ok(Some(((byte >> (7 - bitpointer)) & 1) != 0))
    }
}
