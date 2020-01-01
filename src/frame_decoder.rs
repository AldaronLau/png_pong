use std::{io::Read, marker::PhantomData};
use crate::{Frame, ChunkDecoder, Format, DecodeError};

/// Frame Encoder for PNG files.
pub struct FrameDecoder<R: Read, F: Format> {
    decoder: ChunkDecoder<R>,
    _phantom: PhantomData<F>,
    // FIXME: This is a workaround for not supporting APNG yet.
    has_decoded: bool,
}

impl<R: Read, F: Format> FrameDecoder<R, F> {
    /// Create a new encoder.
    pub fn new(r: R) -> Self {
        FrameDecoder {
            decoder: ChunkDecoder::new(r),
            _phantom: PhantomData,
            has_decoded: false,
        }
    }
}

impl<R, F> Iterator for FrameDecoder<R, F>
where
    R: std::io::Read,
    F: Format<Chan = pix::Ch8>, // FIXME
{
    type Item = Result<Frame<F>, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_decoded {
            return None;
        }
        self.has_decoded = true;

        if cfg!(feature = "crc_checksums") {
            self.decoder.state.decoder.check_crc = true;
        }
        if cfg!(feature = "adler32_checksums") {
            self.decoder.state.decoder.zlibsettings.check_adler32 = true;
        }

        let mut bytes: Vec<u8> = vec![];

        let raster = match self.decoder.bytes.read_to_end(&mut bytes) {
            Ok(_len) => match self.decoder.state.decode(bytes) {
                Ok(raster) => Ok(Frame { raster, delay: 0 }),
                Err(error) => Err(error),
            }
            Err(e) => Err(DecodeError::Io(e))
        };

        Some(raster)
    }
}
