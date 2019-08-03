//! Color types and such

use crate::lodepng::ffi::{ColorMode, ColorType};

pub(super) fn check_png_color_validity(
    colortype: ColorType,
    bd: u32,
) -> Result<(), super::Error> {
    /*allowed color type / bits combination*/
    match colortype {
        ColorType::GREY => {
            if !(bd == 1 || bd == 2 || bd == 4 || bd == 8 || bd == 16) {
                return Err(super::Error(37));
            }
        }
        ColorType::PALETTE => {
            if !(bd == 1 || bd == 2 || bd == 4 || bd == 8) {
                return Err(super::Error(37));
            }
        }
        ColorType::RGB | ColorType::GREY_ALPHA | ColorType::RGBA => {
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
        ColorType::BGRA | ColorType::BGRX | ColorType::BGR if bd == 8 => Ok(()),
        ct => check_png_color_validity(ct, bd),
    }
}

pub fn lodepng_color_mode_equal(a: &ColorMode, b: &ColorMode) -> bool {
    a.colortype == b.colortype
        && a.bitdepth() == b.bitdepth()
        && a.key() == b.key()
        && a.palette() == b.palette()
}
