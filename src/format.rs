use crate::lodepng::ColorType;

/// PNG compatible subset of pix `Format`s.
pub trait Format: pix::Format {
    /// Format to save as.
    const PNG_COLOR: ColorType;
    /// Bit Depth to save as.
    const BIT_DEPTH: u32;
}

impl Format for pix::Gray8 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::Gray16 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::Gray32 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 32;
}

impl Format for pix::GrayAlpha8 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::GrayAlpha16 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::GrayAlpha32 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 32;
}

impl Format for pix::Rgb8 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::Rgb16 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::Rgb32 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 32;
}

impl Format for pix::Rgba8 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::Rgba16 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::Rgba32 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 32;
}
