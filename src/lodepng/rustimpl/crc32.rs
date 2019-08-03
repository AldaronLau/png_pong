//! CRC32

use pix::Alpha;

use super::*;

/* CRC polynomial: 0xedb88320 */
const LODEPNG_CRC32_TABLE: [u32; 256] = [
    0, 1996959894, 3993919788, 2567524794, 124634137, 1886057615, 3915621685,
    2657392035, 249268274, 2044508324, 3772115230, 2547177864, 162941995,
    2125561021, 3887607047, 2428444049, 498536548, 1789927666, 4089016648,
    2227061214, 450548861, 1843258603, 4107580753, 2211677639, 325883990,
    1684777152, 4251122042, 2321926636, 335633487, 1661365465, 4195302755,
    2366115317, 997073096, 1281953886, 3579855332, 2724688242, 1006888145,
    1258607687, 3524101629, 2768942443, 901097722, 1119000684, 3686517206,
    2898065728, 853044451, 1172266101, 3705015759, 2882616665, 651767980,
    1373503546, 3369554304, 3218104598, 565507253, 1454621731, 3485111705,
    3099436303, 671266974, 1594198024, 3322730930, 2970347812, 795835527,
    1483230225, 3244367275, 3060149565, 1994146192, 31158534, 2563907772,
    4023717930, 1907459465, 112637215, 2680153253, 3904427059, 2013776290,
    251722036, 2517215374, 3775830040, 2137656763, 141376813, 2439277719,
    3865271297, 1802195444, 476864866, 2238001368, 4066508878, 1812370925,
    453092731, 2181625025, 4111451223, 1706088902, 314042704, 2344532202,
    4240017532, 1658658271, 366619977, 2362670323, 4224994405, 1303535960,
    984961486, 2747007092, 3569037538, 1256170817, 1037604311, 2765210733,
    3554079995, 1131014506, 879679996, 2909243462, 3663771856, 1141124467,
    855842277, 2852801631, 3708648649, 1342533948, 654459306, 3188396048,
    3373015174, 1466479909, 544179635, 3110523913, 3462522015, 1591671054,
    702138776, 2966460450, 3352799412, 1504918807, 783551873, 3082640443,
    3233442989, 3988292384, 2596254646, 62317068, 1957810842, 3939845945,
    2647816111, 81470997, 1943803523, 3814918930, 2489596804, 225274430,
    2053790376, 3826175755, 2466906013, 167816743, 2097651377, 4027552580,
    2265490386, 503444072, 1762050814, 4150417245, 2154129355, 426522225,
    1852507879, 4275313526, 2312317920, 282753626, 1742555852, 4189708143,
    2394877945, 397917763, 1622183637, 3604390888, 2714866558, 953729732,
    1340076626, 3518719985, 2797360999, 1068828381, 1219638859, 3624741850,
    2936675148, 906185462, 1090812512, 3747672003, 2825379669, 829329135,
    1181335161, 3412177804, 3160834842, 628085408, 1382605366, 3423369109,
    3138078467, 570562233, 1426400815, 3317316542, 2998733608, 733239954,
    1555261956, 3268935591, 3050360625, 752459403, 1541320221, 2607071920,
    3965973030, 1969922972, 40735498, 2617837225, 3943577151, 1913087877,
    83908371, 2512341634, 3803740692, 2075208622, 213261112, 2463272603,
    3855990285, 2094854071, 198958881, 2262029012, 4057260610, 1759359992,
    534414190, 2176718541, 4139329115, 1873836001, 414664567, 2282248934,
    4279200368, 1711684554, 285281116, 2405801727, 4167216745, 1634467795,
    376229701, 2685067896, 3608007406, 1308918612, 956543938, 2808555105,
    3495958263, 1231636301, 1047427035, 2932959818, 3654703836, 1088359270,
    936918000, 2847714899, 3736837829, 1202900863, 817233897, 3183342108,
    3401237130, 1404277552, 615818150, 3134207493, 3453421203, 1423857449,
    601450431, 3009837614, 3294710456, 1567103746, 711928724, 3020668471,
    3272380065, 1510334235, 755167117,
];

/*Return the CRC of the bytes buf[0..len-1].*/
pub fn lodepng_crc32(data: &[u8]) -> u32 {
    let mut r = 4294967295u32;
    for &d in data {
        r = LODEPNG_CRC32_TABLE[((r ^ d as u32) & 255) as usize] ^ (r >> 8);
    }
    r ^ 4294967295
}

impl Drop for Info {
    fn drop(&mut self) {
        self.clear_text();
        self.clear_itext();
        for i in &mut self.unknown_chunks_data {
            i.clear();
        }
    }
}

pub fn lodepng_convert(
    out: &mut [u8],
    inp: &[u8],
    mode_out: &ColorMode,
    mode_in: &ColorMode,
    w: u32,
    h: u32,
) -> Result<(), Error> {
    let numpixels = w as usize * h as usize;
    if lodepng_color_mode_equal(mode_out, mode_in) {
        let numbytes = mode_in.raw_size(w, h);
        out[..numbytes].clone_from_slice(&inp[..numbytes]);
        return Ok(());
    }
    let mut tree = ColorTree::new();
    if mode_out.colortype == ColorType::PALETTE {
        let mut palette = mode_out.palette();
        let palsize = 1 << mode_out.bitdepth();
        /*if the user specified output palette but did not give the values, assume
        they want the values of the input color type (assuming that one is palette).
        Note that we never create a new palette ourselves.*/
        if palette.is_empty() {
            palette = mode_in.palette();
        }
        palette = &palette[0..palette.len().min(palsize)];
        for (i, p) in palette.iter().enumerate() {
            let red = p.red().into();
            let green = p.green().into();
            let blue = p.blue().into();
            let alpha = p.alpha().value().into();

            tree.insert((red, green, blue, alpha), i as u16);
        }
    }
    if mode_in.bitdepth() == 16 && mode_out.bitdepth() == 16 {
        for i in 0..numpixels {
            let (r, g, b, a) = get_pixel_color_rgba16(inp, i, mode_in);
            rgba16_to_pixel(out, i, mode_out, r, g, b, a);
        }
    } else if mode_out.bitdepth() == 8 && mode_out.colortype == ColorType::RGBA
    {
        get_pixel_colors_rgba8(out, numpixels as usize, true, inp, mode_in);
    } else if mode_out.bitdepth() == 8 && mode_out.colortype == ColorType::RGB {
        get_pixel_colors_rgba8(out, numpixels as usize, false, inp, mode_in);
    } else {
        for i in 0..numpixels {
            let (r, g, b, a) = get_pixel_color_rgba8(inp, i, mode_in);
            rgba8_to_pixel(out, i, mode_out, &mut tree, [r, g, b, a])?;
        }
    }
    Ok(())
}

/*out must be buffer big enough to contain full image, and in must contain the full decompressed data from
the IDAT chunks (with filter index bytes and possible padding bits)
return value is error*/
/*
This function converts the filtered-padded-interlaced data into pure 2D image buffer with the PNG's colortype.
Steps:
*) if no Adam7: 1) unfilter 2) remove padding bits (= posible extra bits per scanline if bpp < 8)
*) if adam7: 1) 7x unfilter 2) 7x remove padding bits 3) adam7_deinterlace
NOTE: the in buffer will be overwritten with intermediate data!
*/
pub(super) fn postprocess_scanlines(
    out: &mut [u8],
    inp: &mut [u8],
    w: usize,
    h: usize,
    info_png: &Info,
) -> Result<(), Error> {
    let bpp = info_png.color.bpp() as usize;
    if bpp == 0 {
        return Err(Error(31));
    }
    if info_png.interlace_method == 0 {
        if bpp < 8 && w as usize * bpp != ((w as usize * bpp + 7) / 8) * 8 {
            unfilter_aliased(inp, 0, 0, w, h, bpp)?;
            remove_padding_bits(
                out,
                inp,
                w as usize * bpp,
                ((w as usize * bpp + 7) / 8) * 8,
                h,
            );
        } else {
            unfilter(out, inp, w, h, bpp)?;
        };
    } else {
        let (passw, passh, filter_passstart, padded_passstart, passstart) =
            adam7_get_pass_values(w, h, bpp);
        for i in 0..7 {
            unfilter_aliased(
                inp,
                padded_passstart[i],
                filter_passstart[i],
                passw[i] as usize,
                passh[i] as usize,
                bpp,
            )?;
            if bpp < 8 {
                /*remove padding bits in scanlines; after this there still may be padding
                bits between the different reduced images: each reduced image still starts nicely at a byte*/
                remove_padding_bits_aliased(
                    inp,
                    passstart[i],
                    padded_passstart[i],
                    passw[i] as usize * bpp,
                    ((passw[i] as usize * bpp + 7) / 8) * 8,
                    passh[i] as usize,
                );
            };
        }
        adam7_deinterlace(out, inp, w, h, bpp);
    }
    Ok(())
}

/*
For PNG filter method 0
this function unfilters a single image (e.g. without interlacing this is called once, with Adam7 seven times)
out must have enough bytes allocated already, in must have the scanlines + 1 filter_type byte per scanline
w and h are image dimensions or dimensions of reduced image, bpp is bits per pixel
in and out are allowed to be the same memory address (but aren't the same size since in has the extra filter bytes)
*/
fn unfilter(
    out: &mut [u8],
    inp: &[u8],
    w: usize,
    h: usize,
    bpp: usize,
) -> Result<(), Error> {
    let mut prevline = None;

    /*bytewidth is used for filtering, is 1 when bpp < 8, number of bytes per pixel otherwise*/
    let bytewidth = (bpp + 7) / 8;
    let linebytes = (w * bpp + 7) / 8;
    let in_linebytes = 1 + linebytes; /*the extra filterbyte added to each row*/

    for (out_line, in_line) in out
        .chunks_mut(linebytes)
        .zip(inp.chunks(in_linebytes))
        .take(h)
    {
        let filter_type = in_line[0];
        unfilter_scanline(
            out_line,
            &in_line[1..],
            prevline,
            bytewidth,
            filter_type,
            linebytes,
        )?;
        prevline = Some(out_line);
    }
    Ok(())
}

fn unfilter_aliased(
    inout: &mut [u8],
    out_off: usize,
    in_off: usize,
    w: usize,
    h: usize,
    bpp: usize,
) -> Result<(), Error> {
    let mut prevline = None;
    /*bytewidth is used for filtering, is 1 when bpp < 8, number of bytes per pixel otherwise*/
    let bytewidth = (bpp + 7) / 8;
    let linebytes = (w * bpp + 7) / 8;
    for y in 0..h as usize {
        let outindex = linebytes * y;
        let inindex = (1 + linebytes) * y; /*the extra filterbyte added to each row*/
        let filter_type = inout[in_off + inindex];
        unfilter_scanline_aliased(
            inout,
            out_off + outindex,
            in_off + inindex + 1,
            prevline,
            bytewidth,
            filter_type,
            linebytes,
        )?;
        prevline = Some(out_off + outindex);
    }
    Ok(())
}

/*
For PNG filter method 0
unfilter a PNG image scanline by scanline. when the pixels are smaller than 1 byte,
the filter works byte per byte (bytewidth = 1)
precon is the previous unfiltered scanline, recon the result, scanline the current one
the incoming scanlines do NOT include the filter_type byte, that one is given in the parameter filter_type instead
recon and scanline MAY be the same memory address! precon must be disjoint.
*/
pub(super) fn unfilter_scanline(
    recon: &mut [u8],
    scanline: &[u8],
    precon: Option<&[u8]>,
    bytewidth: usize,
    filter_type: u8,
    length: usize,
) -> Result<(), Error> {
    match filter_type {
        0 => recon.clone_from_slice(scanline),
        1 => {
            recon[0..bytewidth].clone_from_slice(&scanline[0..bytewidth]);
            for i in bytewidth..length {
                recon[i] = scanline[i].wrapping_add(recon[i - bytewidth]);
            }
        }
        2 => {
            if let Some(precon) = precon {
                for i in 0..length {
                    recon[i] = scanline[i].wrapping_add(precon[i]);
                }
            } else {
                recon.clone_from_slice(scanline);
            }
        }
        3 => {
            if let Some(precon) = precon {
                for i in 0..bytewidth {
                    recon[i] = scanline[i].wrapping_add(precon[i] >> 1);
                }
                for i in bytewidth..length {
                    let t = recon[i - bytewidth] as u16 + precon[i] as u16;
                    recon[i] = scanline[i].wrapping_add((t >> 1) as u8);
                }
            } else {
                recon[0..bytewidth].clone_from_slice(&scanline[0..bytewidth]);
                for i in bytewidth..length {
                    recon[i] =
                        scanline[i].wrapping_add(recon[i - bytewidth] >> 1);
                }
            }
        }
        4 => {
            if let Some(precon) = precon {
                for i in 0..bytewidth {
                    recon[i] = scanline[i].wrapping_add(precon[i]);
                }
                for i in bytewidth..length {
                    recon[i] = scanline[i].wrapping_add(paeth_predictor(
                        recon[i - bytewidth] as i16,
                        precon[i] as i16,
                        precon[i - bytewidth] as i16,
                    ));
                }
            } else {
                recon[0..bytewidth].clone_from_slice(&scanline[0..bytewidth]);
                for i in bytewidth..length {
                    recon[i] = scanline[i].wrapping_add(recon[i - bytewidth]);
                }
            }
        }
        _ => return Err(Error(36)),
    }
    Ok(())
}

fn unfilter_scanline_aliased(
    inout: &mut [u8],
    recon: usize,
    scanline: usize,
    precon: Option<usize>,
    bytewidth: usize,
    filter_type: u8,
    length: usize,
) -> Result<(), Error> {
    match filter_type {
        0 => {
            for i in 0..length {
                inout[recon + i] = inout[scanline + i];
            }
        }
        1 => {
            for i in 0..bytewidth {
                inout[recon + i] = inout[scanline + i];
            }
            for i in bytewidth..length {
                inout[recon + i] = inout[scanline + i]
                    .wrapping_add(inout[recon + i - bytewidth]);
            }
        }
        2 => {
            if let Some(precon) = precon {
                for i in 0..length {
                    inout[recon + i] =
                        inout[scanline + i].wrapping_add(inout[precon + i]);
                }
            } else {
                for i in 0..length {
                    inout[recon + i] = inout[scanline + i];
                }
            }
        }
        3 => {
            if let Some(precon) = precon {
                for i in 0..bytewidth {
                    inout[recon + i] = inout[scanline + i]
                        .wrapping_add(inout[precon + i] >> 1);
                }
                for i in bytewidth..length {
                    let t = inout[recon + i - bytewidth] as u16
                        + inout[precon + i] as u16;
                    inout[recon + i] =
                        inout[scanline + i].wrapping_add((t >> 1) as u8);
                }
            } else {
                for i in 0..bytewidth {
                    inout[recon + i] = inout[scanline + i];
                }
                for i in bytewidth..length {
                    inout[recon + i] = inout[scanline + i]
                        .wrapping_add(inout[recon + i - bytewidth] >> 1);
                }
            }
        }
        4 => {
            if let Some(precon) = precon {
                for i in 0..bytewidth {
                    inout[recon + i] =
                        inout[scanline + i].wrapping_add(inout[precon + i]);
                }
                for i in bytewidth..length {
                    inout[recon + i] =
                        inout[scanline + i].wrapping_add(paeth_predictor(
                            inout[recon + i - bytewidth] as i16,
                            inout[precon + i] as i16,
                            inout[precon + i - bytewidth] as i16,
                        ));
                }
            } else {
                for i in 0..bytewidth {
                    inout[recon + i] = inout[scanline + i];
                }
                for i in bytewidth..length {
                    inout[recon + i] = inout[scanline + i]
                        .wrapping_add(inout[recon + i - bytewidth]);
                }
            }
        }
        _ => return Err(Error(36)),
    }
    Ok(())
}

/*
After filtering there are still padding bits if scanlines have non multiple of 8 bit amounts. They need
to be removed (except at last scanline of (Adam7-reduced) image) before working with pure image buffers
for the Adam7 code, the color convert code and the output to the user.
in and out are allowed to be the same buffer, in may also be higher but still overlapping; in must
have >= ilinebits*h bits, out must have >= olinebits*h bits, olinebits must be <= ilinebits
also used to move bits after earlier such operations happened, e.g. in a sequence of reduced images from Adam7
only useful if (ilinebits - olinebits) is a value in the range 1..7
*/
fn remove_padding_bits(
    out: &mut [u8],
    inp: &[u8],
    olinebits: usize,
    ilinebits: usize,
    h: usize,
) {
    let diff = ilinebits - olinebits; /*input and output bit pointers*/
    let mut ibp = 0;
    let mut obp = 0;
    for _ in 0..h {
        for _ in 0..olinebits {
            let bit = read_bit_from_reversed_stream(&mut ibp, inp);
            set_bit_of_reversed_stream(&mut obp, out, bit);
        }
        ibp += diff;
    }
}

fn remove_padding_bits_aliased(
    inout: &mut [u8],
    out_off: usize,
    in_off: usize,
    olinebits: usize,
    ilinebits: usize,
    h: usize,
) {
    let diff = ilinebits - olinebits; /*input and output bit pointers*/
    let mut ibp = 0;
    let mut obp = 0;
    for _ in 0..h {
        for _ in 0..olinebits {
            let bit = read_bit_from_reversed_stream(&mut ibp, &inout[in_off..]);
            set_bit_of_reversed_stream(&mut obp, &mut inout[out_off..], bit);
        }
        ibp += diff;
    }
}

/*
in: non-interlaced image with size w*h
out: the same pixels, but re-ordered according to PNG's Adam7 interlacing, with
 no padding bits between scanlines, but between reduced images so that each
 reduced image starts at a byte.
bpp: bits per pixel
there are no padding bits, not between scanlines, not between reduced images
in has the following size in bits: w * h * bpp.
out is possibly bigger due to padding bits between reduced images
NOTE: comments about padding bits are only relevant if bpp < 8
*/
pub(super) fn adam7_interlace(
    out: &mut [u8],
    inp: &[u8],
    w: usize,
    h: usize,
    bpp: usize,
) {
    let (passw, passh, _, _, passstart) = adam7_get_pass_values(w, h, bpp);
    let bpp = bpp;
    if bpp >= 8 {
        for i in 0..7 {
            let bytewidth = bpp / 8;
            for y in 0..passh[i] as usize {
                for x in 0..passw[i] as usize {
                    let pixelinstart = ((ADAM7_IY[i] as usize
                        + y * ADAM7_DY[i] as usize)
                        * w as usize
                        + ADAM7_IX[i] as usize
                        + x * ADAM7_DX[i] as usize)
                        * bytewidth;
                    let pixeloutstart =
                        passstart[i] + (y * passw[i] as usize + x) * bytewidth;
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
            let olinebits = bpp * w;
            for y in 0..passh[i] as usize {
                for x in 0..passw[i] as usize {
                    let mut ibp = (ADAM7_IY[i] as usize
                        + y * ADAM7_DY[i] as usize)
                        * olinebits
                        + (ADAM7_IX[i] as usize + x * ADAM7_DX[i] as usize)
                            * bpp;
                    let mut obp =
                        (8 * passstart[i]) + (y * ilinebits + x * bpp);
                    for _ in 0..bpp {
                        let bit = read_bit_from_reversed_stream(&mut ibp, inp);
                        set_bit_of_reversed_stream(&mut obp, out, bit);
                    }
                }
            }
        }
    };
}
