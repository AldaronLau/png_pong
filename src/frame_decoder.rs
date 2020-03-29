// png-pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
// Copyright © 2014-2017 Kornel Lesiński
// Copyright © 2005-2016 Lode Vandevenne
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::{ChunkDecoder, DecodeError, Format, Frame};
use std::{io::Read, marker::PhantomData};

/// Frame Encoder for PNG files.
#[derive(Debug)]
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
    R: Read,
    F: Format<Chan = pix::channel::Ch8>, // FIXME
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
            },
            Err(e) => Err(DecodeError::Io(e.kind())),
        };

        Some(raster)
    }
}
