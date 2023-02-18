use crate::{
    adam7,
    bitstream::{BitstreamReader, BitstreamWriter},
    chunk::ImageHeader,
    decode::Error as DecoderError,
    encode::filter,
};

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
    w: u32,
    h: u32,
    header: &ImageHeader,
) -> Result<(), DecoderError> {
    let bpp = header.bpp();
    assert_ne!(bpp, 0);
    if !header.interlace {
        if bpp < 8
            && w as usize * bpp as usize
                != ((w as usize * bpp as usize + 7) / 8) * 8
        {
            unfilter_aliased(inp, 0, 0, w as usize, h as usize, bpp as usize)?;
            remove_padding_bits(
                out,
                inp,
                w as usize * bpp as usize,
                ((w as usize * bpp as usize + 7) / 8) * 8,
                h as usize,
            );
        } else {
            unfilter(out, inp, w, h, bpp)?;
        };
    } else {
        let (passw, passh, filter_passstart, padded_passstart, passstart) =
            adam7::get_pass_values(w, h, bpp);
        for i in 0..7 {
            unfilter_aliased(
                inp,
                padded_passstart[i] as usize,
                filter_passstart[i] as usize,
                passw[i] as usize,
                passh[i] as usize,
                bpp as usize,
            )?;
            if bpp < 8 {
                /*remove padding bits in scanlines; after this there still may be padding
                bits between the different reduced images: each reduced image still starts nicely at a byte*/
                remove_padding_bits_aliased(
                    inp,
                    passstart[i] as usize,
                    padded_passstart[i] as usize,
                    passw[i] as usize * bpp as usize,
                    ((passw[i] as usize * bpp as usize + 7) / 8) * 8,
                    passh[i] as usize,
                );
            };
        }
        adam7::deinterlace(out, inp, w, h, bpp);
    }
    Ok(())
}

fn remove_padding_bits_aliased(
    inout: &mut [u8],
    out_off: usize,
    in_off: usize,
    olinebits: usize,
    ilinebits: usize,
    h: usize,
) {
    let diff = ilinebits - olinebits; /* input and output bit pointers */
    let mut ibp = 0;
    let mut out = Vec::with_capacity(olinebits);
    let mut out_stream = BitstreamWriter::new(&mut out);
    for _ in 0..h {
        for _ in 0..olinebits {
            let bit = {
                let mut in_stream = BitstreamReader::with_bitpointer(
                    std::io::Cursor::new(&inout[in_off..]),
                    ibp,
                )
                .unwrap();
                in_stream.read().unwrap().unwrap()
            };
            out_stream.write(bit).unwrap();
        }
        ibp += olinebits + diff;
    }
    // Copy output buffer into array.
    for (i, byte) in out.iter().cloned().enumerate() {
        inout[out_off + i] = byte;
    }
}

fn unfilter_aliased(
    inout: &mut [u8],
    out_off: usize,
    in_off: usize,
    w: usize,
    h: usize,
    bpp: usize,
) -> Result<(), DecoderError> {
    let mut prevline = None;
    // bytewidth is used for filtering, is 1 when bpp < 8, number of bytes per
    // pixel otherwise
    let bytewidth = (bpp + 7) / 8;
    let linebytes = (w * bpp + 7) / 8;
    for y in 0..h {
        let outindex = linebytes * y;
        let inindex = (1 + linebytes) * y; /* the extra filterbyte added to each row */
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
    let diff = ilinebits - olinebits; /* input and output bit pointers */
    let mut ibp = 0;
    let mut out_buf = Vec::new();
    let mut out_stream = BitstreamWriter::new(&mut out_buf);
    for _ in 0..h {
        for _ in 0..olinebits {
            let bit = {
                let mut in_stream = BitstreamReader::with_bitpointer(
                    std::io::Cursor::new(inp),
                    ibp,
                )
                .unwrap();
                in_stream.read().unwrap().unwrap()
            };
            out_stream.write(bit).unwrap();
        }
        ibp += olinebits + diff;
    }
    // Copy output buffer into array.
    for (i, byte) in out_buf.iter().cloned().enumerate() {
        out[i] = byte;
    }
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
    width: u32,
    height: u32,
    bpp: u8,
) -> Result<(), DecoderError> {
    let mut prevline = None;

    /* bytewidth is used for filtering, is 1 when bpp < 8, number of bytes
     * per pixel otherwise */
    let bytewidth = (bpp as usize + 7) / 8;
    let linebytes = (width as usize * bpp as usize + 7) / 8;
    let in_linebytes = 1 + linebytes; /* the extra filterbyte added to each row */

    for (out_line, in_line) in out
        .chunks_mut(linebytes)
        .zip(inp.chunks(in_linebytes))
        .take(height as usize)
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

fn unfilter_scanline_aliased(
    inout: &mut [u8],
    recon: usize,
    scanline: usize,
    precon: Option<usize>,
    bytewidth: usize,
    filter_type: u8,
    length: usize,
) -> Result<(), DecoderError> {
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
                    inout[recon + i] = inout[scanline + i].wrapping_add(
                        filter::paeth_predictor(
                            inout[recon + i - bytewidth] as i16,
                            inout[precon + i] as i16,
                            inout[precon + i - bytewidth] as i16,
                        ),
                    );
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
        _ => return Err(DecoderError::IllegalFilterType),
    }
    Ok(())
}

/// For PNG filter method 0 unfilter a PNG image scanline by scanline. when the
/// pixels are smaller than 1 byte, the filter works byte per byte
/// (bytewidth = 1) precon is the previous unfiltered scanline, recon the
/// result, scanline the current one the incoming scanlines do NOT include the
/// filter_type byte, that one is given in the parameter filter_type instead
/// recon and scanline MAY be the same memory address! precon must be disjoint.
pub(super) fn unfilter_scanline(
    recon: &mut [u8],
    scanline: &[u8],
    precon: Option<&[u8]>,
    bytewidth: usize,
    filter_type: u8,
    length: usize,
) -> Result<(), DecoderError> {
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
                    recon[i] =
                        scanline[i].wrapping_add(filter::paeth_predictor(
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
        _ => return Err(DecoderError::IllegalFilterType),
    }
    Ok(())
}
