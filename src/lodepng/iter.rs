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
