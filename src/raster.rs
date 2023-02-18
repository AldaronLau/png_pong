use pix::{
    chan::{Ch16, Ch8},
    el::Pixel,
    gray::{Gray8, SGray16, SGray8, SGraya16, SGraya8},
    rgb::{SRgb16, SRgb8, SRgba16, SRgba8},
    Palette, Raster,
};

use crate::chunk::{ColorType, ImageHeader};

/// A Raster of one of the PNG types (all are sRGB gamma).
/// PNGs with less than 8 bits per channel are scaled up to 8 bits per channel.
#[allow(missing_debug_implementations)]
pub enum PngRaster {
    /// 1, 2, 4, 8-bit greyscale
    Gray8(Raster<SGray8>),
    /// 16-bit grayscale
    Gray16(Raster<SGray16>),
    /// 8-bit sRGB
    Rgb8(Raster<SRgb8>),
    /// 16-bit sRGB
    Rgb16(Raster<SRgb16>),
    /// 1, 2, 4, 8-bit sRGB(A) palette
    Palette(Raster<Gray8>, Box<Palette>, Vec<u8>),
    /// 8-bit grayscale with alpha
    Graya8(Raster<SGraya8>),
    /// 16-bit grayscale with alpha
    Graya16(Raster<SGraya16>),
    /// 8-bit sRGB with alpha
    Rgba8(Raster<SRgba8>),
    /// 16-bit sRGB with alpha
    Rgba16(Raster<SRgba16>),
}

impl PngRaster {
    pub(crate) fn header(&self, interlace: bool) -> ImageHeader {
        use PngRaster::*;
        match self {
            Gray8(r) => ImageHeader {
                width: r.width(),
                height: r.height(),
                color_type: ColorType::Grey,
                bit_depth: 8,
                interlace,
            },
            Gray16(r) => ImageHeader {
                width: r.width(),
                height: r.height(),
                color_type: ColorType::Grey,
                bit_depth: 16,
                interlace,
            },
            Rgb8(r) => ImageHeader {
                width: r.width(),
                height: r.height(),
                color_type: ColorType::Rgb,
                bit_depth: 8,
                interlace,
            },
            Rgb16(r) => ImageHeader {
                width: r.width(),
                height: r.height(),
                color_type: ColorType::Rgb,
                bit_depth: 16,
                interlace,
            },
            Palette(r, _pal, _pa) => ImageHeader {
                width: r.width(),
                height: r.height(),
                color_type: ColorType::Palette,
                bit_depth: 8,
                interlace,
            },
            Graya8(r) => ImageHeader {
                width: r.width(),
                height: r.height(),
                color_type: ColorType::GreyAlpha,
                bit_depth: 8,
                interlace,
            },
            Graya16(r) => ImageHeader {
                width: r.width(),
                height: r.height(),
                color_type: ColorType::GreyAlpha,
                bit_depth: 16,
                interlace,
            },
            Rgba8(r) => ImageHeader {
                width: r.width(),
                height: r.height(),
                color_type: ColorType::Rgba,
                bit_depth: 8,
                interlace,
            },
            Rgba16(r) => ImageHeader {
                width: r.width(),
                height: r.height(),
                color_type: ColorType::Rgba,
                bit_depth: 16,
                interlace,
            },
        }
    }
}

impl<P: Pixel> From<PngRaster> for Raster<P>
where
    P::Chan: From<Ch8> + From<Ch16>,
{
    fn from(raster: PngRaster) -> Raster<P> {
        use PngRaster::*;
        match raster {
            Gray8(r) => Raster::with_raster(&r),
            Gray16(r) => Raster::with_raster(&r),
            Rgb8(r) => Raster::with_raster(&r),
            Rgb16(r) => Raster::with_raster(&r),
            Palette(raster, pal, pa) => {
                let mut pixels = Vec::with_capacity(raster.pixels().len());
                for pixel in raster.pixels() {
                    let i: u8 = pixel.one().into();
                    let i = i as usize;
                    let px: SRgb8 = pal.entry(i).unwrap();
                    let px = SRgba8::new(
                        px.one(),
                        px.two(),
                        px.three(),
                        Ch8::new(pa[i]),
                    );
                    pixels.push(px.convert());
                }
                Raster::with_pixels(raster.width(), raster.height(), pixels)
            }
            Graya8(r) => Raster::with_raster(&r),
            Graya16(r) => Raster::with_raster(&r),
            Rgba8(r) => Raster::with_raster(&r),
            Rgba16(r) => Raster::with_raster(&r),
        }
    }
}
