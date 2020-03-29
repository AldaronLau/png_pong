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

use super::ChunkRef;
use crate::lodepng::rustimpl;

pub struct ChunksIter<'a> {
    pub(crate) data: &'a [u8],
}

impl<'a> Iterator for ChunksIter<'a> {
    type Item = ChunkRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let header_len = 12;
        if self.data.len() < header_len {
            return None;
        }

        let len = rustimpl::lodepng_chunk_length(self.data);
        if self.data.len() < len + header_len {
            return None;
        }
        let c = ChunkRef::new(&self.data[0..len + header_len]);
        self.data = rustimpl::lodepng_chunk_next(self.data);
        Some(c)
    }
}
