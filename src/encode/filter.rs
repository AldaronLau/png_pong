//! Algorithms for png "filtering" - A compression algorithm applied before
//! the deflate algorithm.

use crate::{
    chunk::{ColorType, ImageHeader},
    zlib,
};

// FIXME: Move to `encode` module
/// Filter strategy for compression.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FilterStrategy {
    /// Every filter at zero
    Zero,
    /// Use filter that gives minumum sum, as described in the official PNG
    /// filter heuristic.  This is a good default (balance between time to
    /// compress and size).
    MinSum,
    /// Use the filter type that gives smallest Shannon entropy for this
    /// scanline. Depending on the image, this is better or worse than minsum.
    Entropy,
    /// Brute-force-search PNG filters by compressing each filter for each
    /// scanline.  Very slow, and only rarely gives better compression than
    /// MINSUM.
    BruteForce,
}

// FIXME: Not Pub
pub(crate) fn paeth_predictor(a: i16, b: i16, c: i16) -> u8 {
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
        _ => {}
    };
}

/// For PNG filter method 0 out must be a buffer with as size:
/// h + (w * h * bpp + 7) / 8, because there are the scanlines with 1 extra byte
/// per scanline
pub(super) fn filter(
    out: &mut [u8],
    inp: &[u8],
    w: usize,
    h: usize,
    header: &ImageHeader,
    filter_strategy: Option<FilterStrategy>,
    level: u8,
) {
    let color_type = header.color_type;
    let bit_depth = header.bit_depth;

    let bpp = color_type.bpp(bit_depth) as usize;

    /* the width of a scanline in bytes, not including the filter type */
    let linebytes = (w * bpp + 7) / 8;
    /* bytewidth is used for filtering, is 1 when bpp < 8, number of bytes
     * per pixel otherwise */
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
    let strategy = if let Some(strategy) = filter_strategy {
        strategy
    } else if color_type == ColorType::Palette || bit_depth < 8 {
        FilterStrategy::Zero
    } else {
        FilterStrategy::MinSum
    };

    // Shouldn't happen
    assert_ne!(bpp, 0);
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
        FilterStrategy::MinSum => {
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
                    /* check if this is smallest sum (or if type == 0 it's
                     * the first case so always store the values) */
                    if type_ == 0 || sum[type_] < smallest {
                        best_type = type_; /* now fill the out values */
                        smallest = sum[type_];
                    };
                }
                prevline = Some(&inp[(y * linebytes)..]);
                out[y * (linebytes + 1)] = best_type as u8;
                /* the first byte of a scanline will be the filter type */
                for x in 0..linebytes {
                    out[y * (linebytes + 1) + 1 + x] = attempt[best_type][x];
                } /* try the 5 filter types */
            } /* the filter type itself is part of the scanline */
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
                    /* check if this is smallest sum (or if type == 0 it's
                     * the first case so always store the values) */
                    if type_ == 0 || sum[type_] < smallest {
                        best_type = type_; /* now fill the out values */
                        smallest = sum[type_]; /* the first byte of a
                                                * scanline will be the filter
                                                * type */
                    }; /* the extra filterbyte added to each row */
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
            let mut size: [usize; 5] = [0, 0, 0, 0, 0]; /* five filtering attempts, one for each filter type */
            let mut smallest = 0;
            let mut best_type = 0;
            /*use fixed tree on the attempts so that the tree is not adapted to the filter_type on purpose,
            to simulate the true case where the tree is the same for the whole image. Sometimes it gives
            better result with dynamic tree anyway. Using the fixed tree sometimes gives worse, but in rare
            cases better compression. It does make this a bit less slow, so it's worth doing this.*/
            let mut attempt = [
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
                vec![0u8; linebytes],
            ];
            for y in 0..h {
                for type_ in 0..5 {
                    /* it already works good enough by testing a part of the
                     * row */
                    filter_scanline(
                        &mut attempt[type_],
                        &inp[(y * linebytes)..],
                        prevline,
                        linebytes,
                        bytewidth,
                        type_ as u8,
                    );
                    size[type_] = 0;
                    let mut _unused = Vec::new();
                    zlib::compress(&mut _unused, &attempt[type_], level);
                    /* check if this is smallest size (or if type == 0 it's
                     * the first case so always store the values) */
                    if type_ == 0 || size[type_] < smallest {
                        best_type = type_; /* the first byte of a scanline will be the filter
                                            * type */
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
}

#[cfg(test)]
mod tests {
    /*use super::*;

    // FIXME
    #[test]
    fn test_filter() {
        let mut line1 = Vec::with_capacity(1 << 16);
        let mut line2 = Vec::with_capacity(1 << 16);
        for p in 0..256 {
            for q in 0..256 {
                line1.push(q);
                line2.push(p);
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
    }*/
}
