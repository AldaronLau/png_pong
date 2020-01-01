use pix::Raster;
use crate::Format;

/// A Frame
pub struct Frame<F: Format> {
    /// Raster associated with this frame.
    pub raster: Raster<F>,
    /// TODO: Delay associated with this frame.
    pub delay: u32,
}
