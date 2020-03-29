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

//! Color types and such

use crate::lodepng::ffi::{ColorMode, ColorType};

pub(super) fn check_png_color_validity(
    colortype: ColorType,
    bd: u32,
) -> Result<(), super::Error> {
    /*allowed color type / bits combination*/
    match colortype {
        ColorType::Grey => {
            if !(bd == 1 || bd == 2 || bd == 4 || bd == 8 || bd == 16) {
                return Err(super::Error(37));
            }
        }
        ColorType::Palette => {
            if !(bd == 1 || bd == 2 || bd == 4 || bd == 8) {
                return Err(super::Error(37));
            }
        }
        ColorType::Rgb | ColorType::GreyAlpha | ColorType::Rgba => {
            if !(bd == 8 || bd == 16) {
                return Err(super::Error(37));
            }
        }
        _ => return Err(super::Error(31)),
    }
    Ok(())
}
/// Internally BGRA is allowed
pub(super) fn check_lode_color_validity(
    colortype: ColorType,
    bd: u32,
) -> Result<(), super::Error> {
    match colortype {
        ColorType::Bgra | ColorType::Bgrx | ColorType::Bgr if bd == 8 => Ok(()),
        ct => check_png_color_validity(ct, bd),
    }
}

pub(crate) fn lodepng_color_mode_equal(a: &ColorMode, b: &ColorMode) -> bool {
    a.colortype == b.colortype
        && a.bitdepth() == b.bitdepth()
        && a.key() == b.key()
        && a.palette() == b.palette()
}
