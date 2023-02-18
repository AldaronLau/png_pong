// FIXME: API to get Adam7 image before fully loaded (the reason it exists).
// And refactor code to depend on that to get the final image.

use crate::bitstream::BitstreamReader;

/// x start values
const IX: [u32; 7] = [0, 4, 0, 2, 0, 1, 0];
/// y start values
const IY: [u32; 7] = [0, 0, 4, 0, 2, 0, 1];

/// x delta values
const DX: [u32; 7] = [8, 8, 4, 4, 2, 2, 1];
/// y delta values
const DY: [u32; 7] = [8, 8, 8, 4, 4, 2, 2];

type PassW = [u32; 7];
type PassH = [u32; 7];
type FilterPassStart = [u32; 8];
type PaddedPassStart = [u32; 8];
type PassStart = [u32; 8];

pub(crate) fn get_pass_values(
    w: u32,
    h: u32,
    bpp: u8,
) -> (PassW, PassH, FilterPassStart, PaddedPassStart, PassStart) {
    let bpp = bpp as u32;
    let mut passw: [u32; 7] = [0; 7];
    let mut passh: [u32; 7] = [0; 7];
    let mut filter_passstart: [u32; 8] = [0; 8];
    let mut padded_passstart: [u32; 8] = [0; 8];
    let mut passstart: [u32; 8] = [0; 8];

    // The passstart values have 8 values: the 8th one indicates the byte after
    // the end of the 7th (= last) pass
    for i in 0..7 {
        // calculate width and height in pixels of each pass
        passw[i] = (w + DX[i] - IX[i] - 1) / DX[i];
        passh[i] = (h + DY[i] - IY[i] - 1) / DY[i];
        // if passw[i] is 0, it's 0 bytes, not 1 (no filter_type-byte)
        if passw[i] == 0 {
            passh[i] = 0; // only padded at end of reduced image
        }
        // bits padded if needed to fill full byte at end of each scanline
        if passh[i] == 0 {
            passw[i] = 0;
        };
    }
    filter_passstart[0] = 0;
    padded_passstart[0] = 0;
    passstart[0] = 0;
    for i in 0..7 {
        filter_passstart[i + 1] = filter_passstart[i]
            + if passw[i] != 0 && passh[i] != 0 {
                passh[i] * (1 + (passw[i] * bpp + 7) / 8)
            } else {
                0
            };
        padded_passstart[i + 1] =
            padded_passstart[i] + passh[i] * ((passw[i] * bpp + 7) / 8);
        passstart[i + 1] = passstart[i] + (passh[i] * passw[i] * bpp + 7) / 8;
    }
    (passw, passh, filter_passstart, padded_passstart, passstart)
}

/// in: Adam7 interlaced image, with no padding bits between scanlines, but
/// between reduced images so that each reduced image starts at a byte.
/// out: the same pixels, but re-ordered so that they're now a non-interlaced
/// image with size w * h bpp: bits per pixel out has the following size in
/// bits: w * h * bpp.  in is possibly bigger due to padding bits between
/// reduced images.  out must be big enough AND must be 0 everywhere if bpp < 8
/// in the current implementation (because that's likely a little bit faster)
///
/// NOTE: comments about padding bits are only relevant if bpp < 8
pub(crate) fn deinterlace(out: &mut [u8], inp: &[u8], w: u32, h: u32, bpp: u8) {
    let (passw, passh, _, _, passstart) = get_pass_values(w, h, bpp);
    let bpp = bpp as u32;
    if bpp >= 8 {
        for i in 0..7 {
            let bytewidth = bpp / 8;
            for y in 0..passh[i] {
                for x in 0..passw[i] {
                    let pixelinstart = (passstart[i]
                        + (y * passw[i] + x) * bytewidth)
                        as usize;
                    let bytewidth = bytewidth as usize;
                    let pixeloutstart =
                        ((IY[i] + y * DY[i]) * w + IX[i] + x * DX[i]) as usize
                            * bytewidth;

                    out[pixeloutstart..(bytewidth + pixeloutstart)]
                        .clone_from_slice(
                            &inp[pixelinstart..(bytewidth + pixelinstart)],
                        )
                }
            }
        }
    } else {
        for i in 0..7 {
            let ilinebits = bpp * passw[i];
            let olinebits = bpp * w;
            for y in 0..passh[i] {
                for x in 0..passw[i] {
                    let mut obp = ((IY[i] + y * DY[i]) * olinebits
                        + (IX[i] + x * DX[i]) * bpp)
                        as usize;
                    let mut in_stream = BitstreamReader::with_bitpointer(
                        std::io::Cursor::new(inp),
                        ((8 * passstart[i]) + (y * ilinebits + x * bpp))
                            as usize,
                    )
                    .unwrap();
                    for _ in 0..bpp {
                        let bit = in_stream.read().unwrap().unwrap();
                        // note that this function assumes the out buffer is
                        // completely 0, use set_bit_of_reversed_stream
                        // otherwise
                        set_bit_of_reversed_stream0(&mut obp, out, bit);
                    }
                }
            }
        }
    };
}

/// in: non-interlaced image with size w*h
/// out: the same pixels, but re-ordered according to PNG's Adam7 interlacing,
/// with no padding bits between scanlines, but between reduced images so that
/// each reduced image starts at a byte.
/// bpp: bits per pixel there are no padding bits, not between scanlines, not
/// between reduced images.  in has the following size in bits: w * h * bpp.
/// out is possibly bigger due to padding bits between reduced images
///
/// NOTE: comments about padding bits are only relevant if bpp < 8
pub(crate) fn interlace(out: &mut [u8], inp: &[u8], w: u32, h: u32, bpp: u8) {
    let (passw, passh, _, _, passstart) = get_pass_values(w, h, bpp);
    let bpp = bpp as usize;
    if bpp >= 8 {
        for i in 0..7 {
            let bytewidth = bpp / 8;
            for y in 0..passh[i] as usize {
                for x in 0..passw[i] as usize {
                    let pixelinstart = ((IY[i] as usize + y * DY[i] as usize)
                        * w as usize
                        + IX[i] as usize
                        + x * DX[i] as usize)
                        * bytewidth;
                    let pixeloutstart = passstart[i] as usize
                        + (y * passw[i] as usize + x) * bytewidth;
                    out[pixeloutstart..(bytewidth + pixeloutstart)]
                        .clone_from_slice(
                            &inp[pixelinstart..(bytewidth + pixelinstart)],
                        );
                }
            }
        }
    } else {
        for i in 0..7 {
            let ilinebits = bpp * passw[i] as usize;
            let olinebits = bpp * w as usize;
            for y in 0..passh[i] as usize {
                for x in 0..passw[i] as usize {
                    let mut obp =
                        (8 * passstart[i] as usize) + (y * ilinebits + x * bpp);
                    let mut in_stream = BitstreamReader::with_bitpointer(
                        std::io::Cursor::new(inp),
                        (IY[i] as usize + y * DY[i] as usize) * olinebits
                            + (IX[i] as usize + x * DX[i] as usize) * bpp,
                    )
                    .unwrap();
                    for _ in 0..bpp {
                        let bit = in_stream.read().unwrap().unwrap();
                        set_bit_of_reversed_stream(&mut obp, out, bit);
                    }
                }
            }
        }
    };
}

/// Like `set_bit_of_reversed_stream()`, except assumes the current value of the
/// bit is `false`.
#[inline(always)]
pub(crate) fn set_bit_of_reversed_stream0(
    bitpointer: &mut usize,
    bitstream: &mut [u8],
    bit: bool,
) {
    /* the current bit in bitstream must be 0 for this to work */
    if bit {
        /* earlier bit of huffman code is in a lesser significant bit of an
         * earlier byte */
        bitstream[(*bitpointer) >> 3] |= 1 << (7 - ((*bitpointer) & 7));
    }
    *bitpointer += 1;
}

#[inline(always)]
pub(crate) fn set_bit_of_reversed_stream(
    bitpointer: &mut usize,
    bitstream: &mut [u8],
    bit: bool,
) {
    if bit {
        bitstream[(*bitpointer) >> 3] |= 1 << (7 - ((*bitpointer) & 7));
    } else {
        bitstream[(*bitpointer) >> 3] &= !(1 << (7 - ((*bitpointer) & 7)));
    }

    *bitpointer += 1;
}
