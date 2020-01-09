use crate::lodepng::ColorType;

/// PNG compatible subset of pix `Format`s.
pub trait Format: pix::Format {
    /// Format to save as.
    const PNG_COLOR: ColorType;
    /// Bit Depth to save as.
    const BIT_DEPTH: u32;
}

impl Format for pix::SepSGray8 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::SepSGray16 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::SepSGray32 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 32;
}

impl Format for pix::SepSGrayAlpha8 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::SepSGrayAlpha16 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::SepSGrayAlpha32 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 32;
}

impl Format for pix::SepSRgb8 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::SepSRgb16 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::SepSRgb32 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 32;
}

impl Format for pix::SepSRgba8 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::SepSRgba16 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::SepSRgba32 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 32;
}
