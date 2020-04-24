// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
// Copyright © 2014-2017 Kornel Lesiński
// Copyright © 2005-2016 Lode Vandevenne
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::lodepng::ColorType;

use pix::gray::{SGray16, SGray32, SGray8, SGraya16, SGraya32, SGraya8};
use pix::rgb::{SRgb16, SRgb32, SRgb8, SRgba16, SRgba32, SRgba8};

/// PNG compatible subset of pix `Format`s.
pub trait Format: pix::el::Pixel {
    /// Format to save as.
    const PNG_COLOR: ColorType;
    /// Bit Depth to save as.
    const BIT_DEPTH: u32;
}

impl Format for SGray8 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 8;
}

impl Format for SGray16 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 16;
}

impl Format for SGray32 {
    const PNG_COLOR: ColorType = ColorType::Grey;
    const BIT_DEPTH: u32 = 32;
}

impl Format for SGraya8 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 8;
}

impl Format for SGraya16 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 16;
}

impl Format for SGraya32 {
    const PNG_COLOR: ColorType = ColorType::GreyAlpha;
    const BIT_DEPTH: u32 = 32;
}

impl Format for SRgb8 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 8;
}

impl Format for SRgb16 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 16;
}

impl Format for SRgb32 {
    const PNG_COLOR: ColorType = ColorType::Rgb;
    const BIT_DEPTH: u32 = 32;
}

impl Format for SRgba8 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 8;
}

impl Format for SRgba16 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 16;
}

impl Format for SRgba32 {
    const PNG_COLOR: ColorType = ColorType::Rgba;
    const BIT_DEPTH: u32 = 32;
}
