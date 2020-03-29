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

mod adler32;
mod bitmath;
mod chunks;
mod colors;
mod crc32;
mod huffman;
mod png_decoder;
mod zlib;

use self::{adler32::*, bitmath::*};
pub(crate) use self::{
    chunks::*,
    colors::*,
    crc32::*,
    huffman::*,
    png_decoder::*,
    zlib::*,
    zlib::{zlib_compress, zlib_decompress},
};

use super::*;
use ffi::ColorProfile;
use ffi::State;
use ChunkPosition;

use std::collections::HashMap;

/*8 bytes PNG signature, aka the magic bytes*/
fn write_signature(out: &mut Vec<u8>) {
    out.push(137u8);
    out.push(80u8);
    out.push(78u8);
    out.push(71u8);
    out.push(13u8);
    out.push(10u8);
    out.push(26u8);
    out.push(10u8);
}

#[derive(Eq, PartialEq)]
enum PaletteTranslucency {
    Opaque,
    Key,
    Semi,
}

/*
palette must have 4 * palettesize bytes allocated, and given in format RGBARGBARGBARGBA...
returns 0 if the palette is opaque,
returns 1 if the palette has a single color with alpha 0 ==> color key
returns 2 if the palette is semi-translucent.
*/
fn get_palette_translucency(palette: &[SRgba8]) -> PaletteTranslucency {
    let mut key = PaletteTranslucency::Opaque;
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    /*the value of the color with alpha 0, so long as color keying is possible*/
    let mut i = 0;
    while i < palette.len() {
        let byte: u8 = pix::RgbModel::alpha(palette[i]).into();
        if key == PaletteTranslucency::Opaque && byte == 0 {
            r = pix::RgbModel::red(palette[i]).into();
            g = pix::RgbModel::green(palette[i]).into();
            b = pix::RgbModel::blue(palette[i]).into();
            key = PaletteTranslucency::Key;
            i = 0;
            /*restart from beginning, to detect earlier opaque colors with key's value*/
            continue;
        } else if byte != 255 {
            return PaletteTranslucency::Semi;
        } else if key == PaletteTranslucency::Key
            && r == pix::RgbModel::red(palette[i]).into()
            && g == pix::RgbModel::green(palette[i]).into()
            && b == pix::RgbModel::blue(palette[i]).into()
        {
            /*when key, no opaque RGB may have key's RGB*/
            return PaletteTranslucency::Semi;
        }
        i += 1;
    }
    key
}

/*The opposite of the remove_padding_bits function
olinebits must be >= ilinebits*/
fn add_padding_bits(
    out: &mut [u8],
    inp: &[u8],
    olinebits: usize,
    ilinebits: usize,
    h: usize,
) {
    let diff = olinebits - ilinebits; /*bit pointers*/
    let mut obp = 0;
    let mut ibp = 0;
    for _ in 0..h {
        for _ in 0..ilinebits {
            let bit = read_bit_from_reversed_stream(&mut ibp, inp);
            set_bit_of_reversed_stream(&mut obp, out, bit);
        }
        for _ in 0..diff {
            set_bit_of_reversed_stream(&mut obp, out, 0u8);
        }
    }
}

/*out must be buffer big enough to contain uncompressed IDAT chunk data, and in must contain the full image.
return value is error**/
fn pre_process_scanlines(
    inp: &[u8],
    width: u32,
    height: u32,
    info_png: &Info,
    settings: &EncoderSettings,
) -> Result<Vec<u8>, Error> {
    let h = height as usize;
    let w = width as usize;
    /*
    This function converts the pure 2D image with the PNG's colortype, into filtered-padded-interlaced data. Steps:
    *) if no Adam7: 1) add padding bits (= posible extra bits per scanline if bpp < 8) 2) filter
    *) if adam7: 1) adam7_interlace 2) 7x add padding bits 3) 7x filter
    */
    let bpp = info_png.color.bpp() as usize;
    if info_png.interlace_method == 0 {
        let outsize = h + (h * ((w * bpp + 7) / 8));
        let mut out = vec![0u8; outsize];
        /*image size plus an extra byte per scanline + possible padding bits*/
        if bpp < 8 && w * bpp != ((w * bpp + 7) / 8) * 8 {
            let mut padded = vec![0u8; h * ((w * bpp + 7) / 8)]; /*we can immediately filter into the out buffer, no other steps needed*/
            add_padding_bits(
                &mut padded,
                inp,
                ((w * bpp + 7) / 8) * 8,
                w * bpp,
                h,
            );
            filter(&mut out, &padded, w, h, &info_png.color, settings)?;
        } else {
            filter(&mut out, inp, w, h, &info_png.color, settings)?;
        }
        Ok(out)
    } else {
        let (passw, passh, filter_passstart, padded_passstart, passstart) =
            adam7_get_pass_values(width, height, bpp as u32);
        let outsize = filter_passstart[7];
        /*image size plus an extra byte per scanline + possible padding bits*/
        let mut out = vec![0u8; (outsize as usize)];
        let mut adam7 = vec![0u8; passstart[7] as usize + 1];
        adam7_interlace(&mut adam7, inp, width, height, bpp as u32);
        for i in 0..7 {
            if bpp < 8 {
                let mut padded = vec![
                    0u8;
                    (padded_passstart[i + 1] - padded_passstart[i])
                        as usize
                ];
                add_padding_bits(
                    &mut padded,
                    &adam7[passstart[i] as usize..],
                    ((passw[i] as usize * bpp + 7) / 8) * 8,
                    passw[i] as usize * bpp,
                    passh[i] as usize,
                );
                filter(
                    &mut out[filter_passstart[i] as usize..],
                    &padded,
                    passw[i] as usize,
                    passh[i] as usize,
                    &info_png.color,
                    settings,
                )?;
            } else {
                filter(
                    &mut out[filter_passstart[i] as usize..],
                    &adam7[padded_passstart[i] as usize..],
                    passw[i] as usize,
                    passh[i] as usize,
                    &info_png.color,
                    settings,
                )?;
            }
        }
        Ok(out)
    }
}

/*
For PNG filter method 0
out must be a buffer with as size: h + (w * h * bpp + 7) / 8, because there are
the scanlines with 1 extra byte per scanline
*/
fn filter(
    out: &mut [u8],
    inp: &[u8],
    w: usize,
    h: usize,
    info: &ColorMode,
    settings: &EncoderSettings,
) -> Result<(), Error> {
    let bpp = info.bpp() as usize;

    /*the width of a scanline in bytes, not including the filter type*/
    let linebytes = (w * bpp + 7) / 8;
    /*bytewidth is used for filtering, is 1 when bpp < 8, number of bytes per pixel otherwise*/
    let bytewidth = (bpp + 7) / 8;
    let mut prevline = None;
    /*
    There is a heuristic called the minimum sum of absolute differences heuristic, suggested by the PNG standard:
     *  If the image type is Palette, or the bit depth is smaller than 8, then do not filter the image (i.e.
        use fixed filtering, with the filter None).
     * (The other case) If the image type is Grayscale or RGB (with or without Alpha), and the bit depth is
       not smaller than 8, then use adaptive filtering heuristic as follows: independently for each row, apply
       all five filters and select the filter that produces the smallest sum of absolute values per row.
    This heuristic is used if filter strategy is FilterStrategy::MINSUM and filter_palette_zero is true.

    If filter_palette_zero is true and filter_strategy is not FilterStrategy::MINSUM, the above heuristic is followed,
    but for "the other case", whatever strategy filter_strategy is set to instead of the minimum sum
    heuristic is used.
    */
    let strategy = if settings.filter_palette_zero != 0
        && (info.colortype == ColorType::Palette || info.bitdepth() < 8)
    {
        FilterStrategy::Zero
    } else {
        settings.filter_strategy
    };
    if bpp == 0 {
        return Err(Error(31));
    }
    match strategy {
        FilterStrategy::Zero => {
            for y in 0..h {
                let outindex = (1 + linebytes) * y;
                let inindex = linebytes * y;
                out[outindex] = 0u8;
                filter_scanline(
                    &mut out[(outindex + 1)..],
                    &inp[inindex..],
                    prevline,
                    linebytes,
                    bytewidth,
                    0u8,
                );
                prevline = Some(&inp[inindex..]);
            }
        }
        FilterStrategy::Minsum => {
            let mut sum: [usize; 5] = [0, 0, 0, 0, 0];
            let mut attempt = [
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
            ];
            let mut smallest = 0;
            let mut best_type = 0;
            for y in 0..h {
                for type_ in 0..5 {
                    filter_scanline(
                        &mut attempt[type_],
                        &inp[(y * linebytes)..],
                        prevline,
                        linebytes,
                        bytewidth,
                        type_ as u8,
                    );
                    sum[type_] = if type_ == 0 {
                        attempt[type_][0..linebytes]
                            .iter()
                            .map(|&s| s as usize)
                            .sum()
                    } else {
                        /*For differences, each byte should be treated as signed, values above 127 are negative
                        (converted to signed char). filter_type 0 isn't a difference though, so use unsigned there.
                        This means filter_type 0 is almost never chosen, but that is justified.*/
                        attempt[type_][0..linebytes]
                            .iter()
                            .map(
                                |&s| if s < 128 { s } else { 255 - s } as usize,
                            )
                            .sum()
                    };
                    /*check if this is smallest sum (or if type == 0 it's the first case so always store the values)*/
                    if type_ == 0 || sum[type_] < smallest {
                        best_type = type_; /*now fill the out values*/
                        smallest = sum[type_];
                    };
                }
                prevline = Some(&inp[(y * linebytes)..]);
                out[y * (linebytes + 1)] = best_type as u8;
                /*the first byte of a scanline will be the filter type*/
                for x in 0..linebytes {
                    out[y * (linebytes + 1) + 1 + x] = attempt[best_type][x];
                } /*try the 5 filter types*/
            } /*the filter type itself is part of the scanline*/
        }
        FilterStrategy::Entropy => {
            let mut sum: [f32; 5] = [0., 0., 0., 0., 0.];
            let mut smallest = 0.;
            let mut best_type = 0;
            let mut attempt = [
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
            ];
            for y in 0..h {
                for type_ in 0..5 {
                    filter_scanline(
                        &mut attempt[type_],
                        &inp[(y * linebytes)..],
                        prevline,
                        linebytes,
                        bytewidth,
                        type_ as u8,
                    );
                    let mut count: [u32; 256] = [0; 256];
                    for x in 0..linebytes {
                        count[attempt[type_][x] as usize] += 1;
                    }
                    count[type_] += 1;
                    sum[type_] = 0.;
                    for &c in count.iter() {
                        let p = c as f32 / ((linebytes + 1) as f32);
                        sum[type_] +=
                            if c == 0 { 0. } else { (1. / p).log2() * p };
                    }
                    /*check if this is smallest sum (or if type == 0 it's the first case so always store the values)*/
                    if type_ == 0 || sum[type_] < smallest {
                        best_type = type_; /*now fill the out values*/
                        smallest = sum[type_]; /*the first byte of a scanline will be the filter type*/
                    }; /*the extra filterbyte added to each row*/
                }
                prevline = Some(&inp[(y * linebytes)..]);
                out[y * (linebytes + 1)] = best_type as u8;
                for x in 0..linebytes {
                    out[y * (linebytes + 1) + 1 + x] = attempt[best_type][x];
                }
            }
        }
        FilterStrategy::BruteForce => {
            /*brute force filter chooser.
            deflate the scanline after every filter attempt to see which one deflates best.
            This is very slow and gives only slightly smaller, sometimes even larger, result*/
            let mut size: [usize; 5] = [0, 0, 0, 0, 0]; /*five filtering attempts, one for each filter type*/
            let mut smallest = 0;
            let mut best_type = 0;
            let mut zlibsettings = settings.zlibsettings.clone();
            /*use fixed tree on the attempts so that the tree is not adapted to the filter_type on purpose,
            to simulate the true case where the tree is the same for the whole image. Sometimes it gives
            better result with dynamic tree anyway. Using the fixed tree sometimes gives worse, but in rare
            cases better compression. It does make this a bit less slow, so it's worth doing this.*/
            zlibsettings.btype = 1;
            /*a custom encoder likely doesn't read the btype setting and is optimized for complete PNG
            images only, so disable it*/
            let mut attempt = [
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
            ];
            for y in 0..h {
                for type_ in 0..5 {
                    /*it already works good enough by testing a part of the row*/
                    filter_scanline(
                        &mut attempt[type_],
                        &inp[(y * linebytes)..],
                        prevline,
                        linebytes,
                        bytewidth,
                        type_ as u8,
                    );
                    size[type_] = 0;
                    let _ = zlib_compress(&attempt[type_], &zlibsettings)?;
                    /*check if this is smallest size (or if type == 0 it's the first case so always store the values)*/
                    if type_ == 0 || size[type_] < smallest {
                        best_type = type_; /*the first byte of a scanline will be the filter type*/
                        smallest = size[type_]; /* unknown filter strategy */
                    }
                }
                prevline = Some(&inp[(y * linebytes)..]);
                out[y * (linebytes + 1)] = best_type as u8;
                for x in 0..linebytes {
                    out[y * (linebytes + 1) + 1 + x] = attempt[best_type][x];
                }
            }
        }
    };
    Ok(())
}

#[test]
fn test_filter() {
    let mut line1 = Vec::with_capacity(1 << 16);
    let mut line2 = Vec::with_capacity(1 << 16);
    for p in 0..256 {
        for q in 0..256 {
            line1.push(q as u8);
            line2.push(p as u8);
        }
    }

    let mut filtered = vec![99u8; 1 << 16];
    let mut unfiltered = vec![66u8; 1 << 16];
    for filter_type in 0..5 {
        let len = filtered.len();
        filter_scanline(
            &mut filtered,
            &line1,
            Some(&line2),
            len,
            1,
            filter_type,
        );
        unfilter_scanline(
            &mut unfiltered,
            &filtered,
            Some(&line2),
            1,
            filter_type,
            len,
        )
        .unwrap();
        assert_eq!(unfiltered, line1, "prev+filter={}", filter_type);
    }
    for filter_type in 0..5 {
        let len = filtered.len();
        filter_scanline(&mut filtered, &line1, None, len, 1, filter_type);
        unfilter_scanline(
            &mut unfiltered,
            &filtered,
            None,
            1,
            filter_type,
            len,
        )
        .unwrap();
        assert_eq!(unfiltered, line1, "none+filter={}", filter_type);
    }
}

fn filter_scanline(
    out: &mut [u8],
    scanline: &[u8],
    prevline: Option<&[u8]>,
    length: usize,
    bytewidth: usize,
    filter_type: u8,
) {
    match filter_type {
        0 => {
            out[..length].clone_from_slice(&scanline[..length]);
        }
        1 => {
            out[..bytewidth].clone_from_slice(&scanline[..bytewidth]);
            for i in bytewidth..length {
                out[i] = scanline[i].wrapping_sub(scanline[i - bytewidth]);
            }
        }
        2 => {
            if let Some(prevline) = prevline {
                for i in 0..length {
                    out[i] = scanline[i].wrapping_sub(prevline[i]);
                }
            } else {
                out[..length].clone_from_slice(&scanline[..length]);
            }
        }
        3 => {
            if let Some(prevline) = prevline {
                for i in 0..bytewidth {
                    out[i] = scanline[i].wrapping_sub(prevline[i] >> 1);
                }
                for i in bytewidth..length {
                    let s = scanline[i - bytewidth] as u16 + prevline[i] as u16;
                    out[i] = scanline[i].wrapping_sub((s >> 1) as u8);
                }
            } else {
                out[..bytewidth].clone_from_slice(&scanline[..bytewidth]);
                for i in bytewidth..length {
                    out[i] =
                        scanline[i].wrapping_sub(scanline[i - bytewidth] >> 1);
                }
            }
        }
        4 => {
            if let Some(prevline) = prevline {
                for i in 0..bytewidth {
                    out[i] = scanline[i].wrapping_sub(prevline[i]);
                }
                for i in bytewidth..length {
                    out[i] = scanline[i].wrapping_sub(paeth_predictor(
                        scanline[i - bytewidth].into(),
                        prevline[i].into(),
                        prevline[i - bytewidth].into(),
                    ));
                }
            } else {
                out[..bytewidth].clone_from_slice(&scanline[..bytewidth]);
                for i in bytewidth..length {
                    out[i] = scanline[i].wrapping_sub(scanline[i - bytewidth]);
                }
            }
        }
        _ => return,
    };
}

fn paeth_predictor(a: i16, b: i16, c: i16) -> u8 {
    let pa = (b - c).abs();
    let pb = (a - c).abs();
    let pc = (a + b - c - c).abs();
    if pc < pa && pc < pb {
        c as u8
    } else if pb < pa {
        b as u8
    } else {
        a as u8
    }
}

pub(crate) fn lodepng_get_bpp_lct(colortype: ColorType, bitdepth: u32) -> u32 {
    assert!(bitdepth >= 1 && bitdepth <= 16);
    /*bits per pixel is amount of channels * bits per channel*/
    let ch = colortype.channels() as u32;
    ch * if ch > 1 {
        if bitdepth == 8 {
            8
        } else {
            16
        }
    } else {
        bitdepth
    }
}
