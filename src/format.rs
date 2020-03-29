use crate::lodepng::ColorType;

/// PNG compatible subset of pix `Format`s.
pub trait Format: pix::Pixel {
    /// Format to save as.
    const PNG_COLOR: ColorType;
    /// Bit Depth to save as.
    const BIT_DEPTH: u32;
}

impl Format for pix::SGray8 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::SGray16 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::SGray32 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 32;
}

impl Format for pix::SGraya8 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::SGraya16 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::SGraya32 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 32;
}

impl Format for pix::SRgb8 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::SRgb16 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::SRgb32 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 32;
}

impl Format for pix::SRgba8 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 8;
}

impl Format for pix::SRgba16 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 16;
}

impl Format for pix::SRgba32 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 32;
}
