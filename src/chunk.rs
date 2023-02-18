//! Low-level PNG API
//!
//! A PNG file consists of a sequence of [`Chunk`](enum.Chunk.html)s in a
//! specific order.
//!
//! # PNG Chunk Order
//! ## Key
//! - **Required** - Count must be exactly one.
//! - **Optional** - Count must be exactly one or zero.
//! - **Multiple** - Count can be any number, including zero.
//!
//! ## Order
//! The PNG/APNG chunk order must be as follows:
//!
//! - **Required** `ImageHeader` "IHDR"
//! - In any order:
//!   - **Optional** `Chromaticities`
//!   - **Optional** `Gamma` "gAMA"
//!   - **Optional** `ColorProfile` "iCCP"
//!   - **Optional** `SignificantBits` "sBIT"
//!   - **Optional** `SRgb` "sRGB"
//!   - **Optional** `Physical` "pHYs"
//!   - **Multiple** `SuggestedPalette` "sPLT"
//!   - **Optional** `Time` "tIME" (If didn't appear earlier)
//!   - **Multiple** `InternationalText` "iTXt"
//!   - **Multiple** `Text` "tEXt"
//!   - **Multiple** `CompressedText` "zTXt"
//!   - **Optional** `AnimationControl` "acTL" (APNG)
//!   - **Optional** `FrameControl` "fcTL" (APNG)
//!   - **Optional** `ImageOffset` "oFFs" (*Extension*)
//!   - **Optional** `PixelCalibration` "pCAL" (*Extension*)
//!   - **Optional** `SubjectPhysical` "sCAL" (*Extension*)
//!   - **Multiple** `GifGraphicControlExt` "gIFg" (*Extension*)
//!   - **Multiple** `GifApplicationExt` "gIFx" (*Extension*)
//! - **Optional** `Palette` "PLTE"
//! - In any order:
//!   - **Optional** `Background` "bKGD"
//!   - **Optional** `PaletteHistogram` "hIST"
//!   - **Optional** `Transparency` "tRNS"
//!   - **Optional** `Physical` "pHYs" (If didn't appear before PLTE)
//!   - **Multiple** `SuggestedPalette` "sPLT"
//!   - **Optional** `Time` "tIME" (If didn't appear earlier)
//!   - **Multiple** `InternationalText` "iTXt"
//!   - **Multiple** `Text` "tEXt"
//!   - **Multiple** `CompressedText` "zTXt"
//!   - **Optional** `AnimationControl` "acTL" (APNG, If didn't appear earlier)
//!   - **Optional** `FrameControl` "fcTL" (APNG, If didn't appear earlier)
//!   - **Optional** `ImageOffset` "oFFs" (*Extension*, If didn't appear
//!     earlier)
//!   - **Optional** `PixelCalibration` "pCAL" (*Extension*, If didn't appear
//!     earlier)
//!   - **Optional** `SubjectPhysical` "sCAL" (*Extension*, If didn't appear
//!     earlier)
//!   - **Multiple** `GifGraphicControlExt` "gIFg" (*Extension*)
//!   - **Multiple** `GifApplicationExt` "gIFx" (*Extension*)
//! - **Multiple** `ImageData` "IDAT"
//! - In any order:
//!   - **Optional** `Time` "tIME" (If didn't appear earlier)
//!   - **Multiple** `InternationalText` "iTXt"
//!   - **Multiple** `Text` "tEXt"
//!   - **Multiple** `CompressedText` "zTXt"
//!   - **Multiple** `FrameControl` "fcTL" (APNG)
//!   - **Multiple** `FrameData` "fdAT" (APNG, must be somewhere after "fcTL")
//!   - **Multiple** `GifGraphicControlExt` "gIFg" (*Extension*)
//!   - **Multiple** `GifApplicationExt` "gIFx" (*Extension*)
//! - **Required** `ImageEnd` "IEND"

use crate::{
    decode::{Error as DecoderError, Result as DecoderResult},
    encode::{Error as EncoderError, Result as EncoderResult},
};

mod bkgd;
mod idat;
mod iend;
mod ihdr;
mod itxt;
mod phys;
mod plte;
mod text;
mod time;
mod trns;
mod unknown;
mod ztxt;

pub use self::{
    // Optional
    bkgd::Background,
    // Required
    idat::ImageData,
    // Required
    iend::ImageEnd,
    // Required
    ihdr::{ColorType, ImageHeader},
    // Optional
    itxt::InternationalText,
    // Optional
    phys::Physical,
    // Required
    plte::Palette,
    // Optional
    text::Text,
    // Optional
    time::Time,
    // Optional
    trns::Transparency,
    // Optional
    unknown::Unknown,
    // Optional
    ztxt::CompressedText,
};

/// A chunk within a PNG file.
#[derive(Debug)]
pub enum Chunk {
    /// Required: Image Header
    ImageHeader(ImageHeader),
    /// Required: Image Data
    ImageData(ImageData),
    /// Required: Image End
    ImageEnd(ImageEnd),

    /// Maybe Required: Palette chunk.
    Palette(Palette),

    /// Optional: Background color chunk.
    Background(Background),
    /// Optional: International text chunk.
    InternationalText(InternationalText),
    /// Optional: Physical dimensions chunk
    Physical(Physical),
    /// Optional: Non-International text chunk.
    Text(Text),
    /// Optional: Time chunk.
    Time(Time),
    /// Optional: Alpha palette chunk.
    Transparency(Transparency),
    /// Optional: Z text chunk.
    CompressedText(CompressedText),
    /// Unknown chunk
    Unknown(Unknown),
}

impl Chunk {
    pub(super) fn is_idat(&self) -> bool {
        matches!(self, Chunk::ImageData(_))
    }

    pub(super) fn is_iend(&self) -> bool {
        matches!(self, Chunk::ImageEnd(_))
    }
}
