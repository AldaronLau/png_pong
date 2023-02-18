use crate::PngRaster;

/// A Frame
pub struct Step {
    /// Raster associated with this frame.
    pub raster: PngRaster,
    /// TODO: Delay associated with this frame.
    pub delay: u32,
}

impl std::fmt::Debug for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.delay)
    }
}
