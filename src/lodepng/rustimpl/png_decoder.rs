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

//! PNG Decoder

use super::*;

/*read the information from the header and store it in the Info. return value is error*/
pub(crate) fn lodepng_inspect(
    decoder: &DecoderSettings,
    inp: &[u8],
) -> Result<(Info, u32, u32), Error> {
    if inp.len() < 33 {
        /*error: the data length is smaller than the length of a PNG header*/
        return Err(Error(27));
    }
    /*when decoding a new PNG image, make sure all parameters created after previous decoding are reset*/
    let mut info_png = Info::new();
    if inp[0..8] != [137, 80, 78, 71, 13, 10, 26, 10] {
        /*error: the first 8 bytes are not the correct PNG signature*/
        return Err(Error(28));
    }
    if lodepng_chunk_length(&inp[8..]) != 13 {
        /*error: header size must be 13 bytes*/
        return Err(Error(94));
    }
    if lodepng_chunk_type(&inp[8..]) != b"IHDR" {
        /*error: it doesn't start with a IHDR chunk!*/
        return Err(Error(29));
    }
    /*read the values given in the header*/
    let w = lodepng_read32bit_int(&inp[16..]);
    let h = lodepng_read32bit_int(&inp[20..]);
    let bitdepth = inp[24];
    if bitdepth == 0 || bitdepth > 16 {
        return Err(Error(29));
    }
    info_png.color.set_bitdepth(inp[24] as u32);
    info_png.color.colortype = match inp[25] {
        0 => ColorType::Grey,
        2 => ColorType::Rgb,
        3 => ColorType::Palette,
        4 => ColorType::GreyAlpha,
        6 => ColorType::Rgba,
        _ => return Err(Error(31)),
    };
    info_png.compression_method = inp[26] as u32;
    info_png.filter_method = inp[27] as u32;
    info_png.interlace_method = inp[28] as u32;
    if w == 0 || h == 0 {
        return Err(Error(93));
    }
    if decoder.check_crc {
        let crc = lodepng_read32bit_int(&inp[29..]);
        let checksum = lodepng_crc32(&inp[12..(12 + 17)]);
        if crc != checksum {
            return Err(Error(57));
        };
    }
    if info_png.compression_method != 0 {
        /*error: only compression method 0 is allowed in the specification*/
        return Err(Error(32));
    }
    if info_png.filter_method != 0 {
        /*error: only filter method 0 is allowed in the specification*/
        return Err(Error(33));
    }
    if info_png.interlace_method > 1 {
        /*error: only interlace methods 0 and 1 exist in the specification*/
        return Err(Error(34));
    }
    check_png_color_validity(
        info_png.color.colortype,
        info_png.color.bitdepth(),
    )?;
    Ok((info_png, w, h))
}

/*read a PNG, the result will be in the same color type as the PNG (hence "generic")*/
pub(super) fn decode_generic(
    state: &mut State,
    inp: &[u8],
) -> Result<(Vec<u8>, u32, u32), Error> {
    let mut found_iend = false; /*the data from idat chunks*/
    /*for unknown chunk order*/
    let mut unknown = false;
    let mut critical_pos = ChunkPosition::IHDR;
    /*provide some proper output values if error will happen*/
    let (info, w, h) = lodepng_inspect(&state.decoder, inp)?;
    state.info_png = info;

    /*reads header and resets other parameters in state->info_png*/
    let numpixels = match w.checked_mul(h) {
        Some(n) => n,
        None => {
            return Err(Error(92));
        }
    };
    /*multiplication overflow possible further below. Allows up to 2^31-1 pixel
    bytes with 16-bit RGBA, the rest is room for filter bytes.*/
    if numpixels > 268435455 {
        return Err(Error(92)); /*first byte of the first chunk after the header*/
    }
    let mut idat = Vec::new();
    let mut chunk = &inp[33..];
    /*loop through the chunks, ignoring unknown chunks and stopping at IEND chunk.
    IDAT data is put at the start of the in buffer*/
    while !found_iend {
        if chunk.len() < 12 {
            return Err(Error(30));
        }
        /*length of the data of the chunk, excluding the length bytes, chunk type and CRC bytes*/
        let data = lodepng_chunk_data(chunk)?;
        match lodepng_chunk_type(chunk) {
            b"IDAT" => {
                idat.extend_from_slice(data);
                critical_pos = ChunkPosition::IDAT;
            }
            b"IEND" => {
                found_iend = true;
            }
            b"PLTE" => {
                read_chunk_plte(&mut state.info_png.color, data)?;
                critical_pos = ChunkPosition::PLTE;
            }
            b"tRNS" => {
                read_chunk_trns(&mut state.info_png.color, data)?;
            }
            b"bKGD" => {
                read_chunk_bkgd(&mut state.info_png, data)?;
            }
            b"tEXt" => {
                // Uncompressed TEXT chunk.
                read_chunk_text(&mut state.info_png, data)?;
            }
            b"zTXt" => {
                // Compressed TEXT chunk.
                read_chunk_ztxt(
                    &mut state.info_png,
                    &state.decoder.zlibsettings,
                    data,
                )?;
            }
            b"iTXt" => {
                // International TEXT chunk.
                read_chunk_itxt(
                    &mut state.info_png,
                    &state.decoder.zlibsettings,
                    data,
                )?;
            }
            b"tIME" => {
                read_chunk_time(&mut state.info_png, data)?;
            }
            b"pHYs" => {
                read_chunk_phys(&mut state.info_png, data)?;
            }
            _ => {
                // An unknown chunk.
                if !lodepng_chunk_ancillary(chunk) {
                    return Err(Error(69));
                }
                unknown = true;
                state.info_png.push_unknown_chunk(critical_pos, chunk)?;
            }
        };
        if state.decoder.check_crc
            && !unknown
            && !lodepng_chunk_check_crc(chunk)
        {
            return Err(Error(57));
        }
        if !found_iend {
            chunk = lodepng_chunk_next(chunk);
        }
    }
    /*predict output size, to allocate exact size for output buffer to avoid more dynamic allocation.
    If the decompressed size does not match the prediction, the image must be corrupt.*/
    let predict = if state.info_png.interlace_method == 0 {
        /*The extra *h is added because this are the filter bytes every scanline starts with*/
        state.info_png.color.raw_size_idat(w, h) + h
    } else {
        /*Adam-7 interlaced: predicted size is the sum of the 7 sub-images sizes*/
        let color = &state.info_png.color;
        let mut predict = color.raw_size_idat((w + 7) >> 3, (h + 7) >> 3)
            + ((h + 7) >> 3);
        if w > 4 {
            predict += color.raw_size_idat((w + 3) >> 3, (h + 7) >> 3)
                + ((h + 7) >> 3);
        }
        predict += color.raw_size_idat((w + 3) >> 2, (h + 3) >> 3)
            + ((h + 3) >> 3);
        if w > 2 {
            predict += color.raw_size_idat((w + 1) >> 2, (h + 3) >> 2)
                + ((h + 3) >> 2);
        }
        predict += color.raw_size_idat((w + 1) >> 1, (h + 1) >> 2)
            + ((h + 1) >> 2);
        if w > 1 {
            predict += color.raw_size_idat((w + 0) >> 1, (h + 1) >> 1)
                + ((h + 1) >> 1);
        }
        predict +=
            color.raw_size_idat(w + 0, (h + 0) >> 1) + ((h + 0) >> 1);
        predict
    };
    let mut scanlines = zlib_decompress(&idat, &state.decoder.zlibsettings)?;
    if scanlines.len() != predict as usize {
        /*decompressed size doesn't match prediction*/
        return Err(Error(91));
    }
    let mut out = Vec::new();
    out.resize(state.info_png.color.raw_size(w, h), 0);
    postprocess_scanlines(&mut out, &mut scanlines, w, h, &state.info_png)?;
    Ok((out, w, h))
}

pub(crate) fn lodepng_decode(
    state: &mut State,
    inp: &[u8],
) -> Result<(Vec<u8>, u32, u32), Error> {
    let (decoded, w, h) = decode_generic(state, inp)?;

    if state.decoder.color_convert == 0
        || lodepng_color_mode_equal(&state.info_raw, &state.info_png.color)
    {
        /*store the info_png color settings on the info_raw so that the info_raw still reflects what colortype
        the raw image has to the end user*/
        if state.decoder.color_convert == 0 {
            /*color conversion needed; sort of copy of the data*/
            state.info_raw = state.info_png.color.clone();
        }
        Ok((decoded, w, h))
    } else {
        /*TODO: check if this works according to the statement in the documentation: "The converter can convert
        from greyscale input color type, to 8-bit greyscale or greyscale with alpha"*/
        if !(state.info_raw.colortype == ColorType::Rgb
            || state.info_raw.colortype == ColorType::Rgba)
            && (state.info_raw.bitdepth() != 8)
        {
            return Err(Error(56)); /*unsupported color mode conversion*/
        }
        let mut out = Vec::new();
        out.resize(state.info_raw.raw_size(w, h), 0);
        lodepng_convert(
            &mut out,
            &decoded,
            &state.info_raw,
            &state.info_png.color,
            w,
            h,
        )?;
        Ok((out, w, h))
    }
}

pub(crate) fn lodepng_decode_memory(
    inp: &[u8],
    colortype: ColorType,
    bitdepth: u32,
) -> Result<(Vec<u8>, u32, u32), Error> {
    let mut state = State::new();
    state.info_raw.colortype = colortype;
    state.info_raw.set_bitdepth(bitdepth);
    lodepng_decode(&mut state, inp)
}

fn add_unknown_chunks(
    out: &mut Vec<u8>,
    mut inchunk: &[u8],
) -> Result<(), Error> {
    while !inchunk.is_empty() {
        chunk_append(out, inchunk);
        inchunk = lodepng_chunk_next(inchunk);
    }
    Ok(())
}

pub(crate) fn lodepng_encode(
    image: &[u8],
    width: u32,
    height: u32,
    state: &mut State,
) -> Result<Vec<u8>, Error> {
    let w = width as usize;
    let h = height as usize;

    let mut info = state.info_png.clone();
    if (info.color.colortype == ColorType::Palette
        || state.encoder.force_palette != 0)
        && (info.color.palette().is_empty() || info.color.palette().len() > 256)
    {
        return Err(Error(68));
    }
    if state.encoder.auto_convert != 0 {
        /*write signature and chunks*/
        info.color = auto_choose_color(image, w, h, &state.info_raw)?;
    }
    if state.encoder.zlibsettings.btype > 2 {
        /*bKGD (must come between PLTE and the IDAt chunks*/
        return Err(Error(61)); /*PLTE*/
    } /*pHYs (must come before the IDAT chunks)*/
    if state.info_png.interlace_method > 1 {
        return Err(Error(71)); /*unknown chunks between PLTE and IDAT*/
        /*IDAT (multiple IDAT chunks must be consecutive)*/
    }
    check_png_color_validity(info.color.colortype, info.color.bitdepth())?; /*tEXt and/or zTXt */
    check_lode_color_validity(
        state.info_raw.colortype,
        state.info_raw.bitdepth(),
    )?; /*LodePNG version id in text chunk */

    let data = if !lodepng_color_mode_equal(&state.info_raw, &info.color) {
        let size = (w * h * (info.color.bpp() as usize) + 7) / 8;
        let mut converted = vec![0u8; size];
        lodepng_convert(
            &mut converted,
            image,
            &info.color,
            &state.info_raw,
            width,
            height,
        )?;
        pre_process_scanlines(&converted, width, height, &info, &state.encoder)?
    } else {
        pre_process_scanlines(image, width, height, &info, &state.encoder)?
    };

    let mut outv = Vec::new();
    write_signature(&mut outv);

    add_chunk_ihdr(
        &mut outv,
        w,
        h,
        info.color.colortype,
        info.color.bitdepth() as usize,
        info.interlace_method as u8,
    )?;
    if let Some(chunks) = info.unknown_chunks_data(ChunkPosition::IHDR) {
        add_unknown_chunks(&mut outv, chunks)?;
    }
    if info.color.colortype == ColorType::Palette {
        add_chunk_plte(&mut outv, &info.color)?;
    }
    if state.encoder.force_palette != 0
        && (info.color.colortype == ColorType::Rgb
            || info.color.colortype == ColorType::Rgba)
    {
        add_chunk_plte(&mut outv, &info.color)?;
    }
    if info.color.colortype == ColorType::Palette
        && get_palette_translucency(info.color.palette())
            != PaletteTranslucency::Opaque
    {
        add_chunk_trns(&mut outv, &info.color)?;
    }
    if (info.color.colortype == ColorType::Grey
        || info.color.colortype == ColorType::Rgb)
        && info.color.key().is_some()
    {
        add_chunk_trns(&mut outv, &info.color)?;
    }
    if info.background_defined != 0 {
        add_chunk_bkgd(&mut outv, &info)?;
    }
    if info.phys_defined != 0 {
        add_chunk_phys(&mut outv, &info)?;
    }
    if let Some(chunks) = info.unknown_chunks_data(ChunkPosition::PLTE) {
        add_unknown_chunks(&mut outv, chunks)?;
    }
    add_chunk_idat(&mut outv, &data, &state.encoder.zlibsettings)?;
    if info.time_defined != 0 {
        add_chunk_time(&mut outv, &info.time)?;
    }
    for ntext in info.text_keys_cstr() {
        if ntext.key.len() > 79 {
            return Err(Error(66));
        }
        if ntext.key.is_empty() {
            return Err(Error(67));
        }
        if state.encoder.text_compression != 0 {
            add_chunk_ztxt(
                &mut outv,
                &ntext.key,
                &ntext.val,
                &state.encoder.zlibsettings,
            )?;
        } else {
            add_chunk_text(&mut outv, &ntext.key, &ntext.val)?;
        }
    }
    if state.encoder.add_id != 0 {
        let alread_added_id_text =
            info.text_keys_cstr().any(|a| a.key == "LodePNG");
        if !alread_added_id_text {
            /*it's shorter as tEXt than as zTXt chunk*/
            let l = "LodePNG";
            let v = "LODEPNG_VERSION_STRING";
            add_chunk_text(&mut outv, l, v)?;
        }
    }
    for chunk in info.itext_keys() {
        if chunk.key.len() > 79 {
            return Err(Error(66));
        }
        if chunk.key.is_empty() {
            return Err(Error(67));
        }
        add_chunk_itxt(
            &mut outv,
            state.encoder.text_compression != 0,
            &chunk.key,
            &chunk.langtag,
            &chunk.transkey,
            &chunk.val,
            &state.encoder.zlibsettings,
        )?;
    }
    if let Some(chunks) = info.unknown_chunks_data(ChunkPosition::IDAT) {
        add_unknown_chunks(&mut outv, chunks)?;
    }
    add_chunk_iend(&mut outv)?;
    Ok(outv)
}

/*profile must already have been inited with mode.
It's ok to set some parameters of profile to done already.*/
pub(crate) fn get_color_profile(
    inp: &[u8],
    w: u32,
    h: u32,
    mode: &ColorMode,
) -> Result<ColorProfile, Error> {
    let mut profile = ColorProfile::new();
    let numpixels: usize = w as usize * h as usize;
    let mut colored_done = mode.is_greyscale_type();
    let mut alpha_done = !mode.can_have_alpha();
    let mut numcolors_done = false;
    let bpp = mode.bpp() as usize;
    let mut bits_done = bpp == 1;
    let maxnumcolors = match bpp {
        1 => 2,
        2 => 4,
        4 => 16,
        5..=8 => 256,
        _ => 257,
    };

    /*Check if the 16-bit input is truly 16-bit*/
    let mut sixteen = false;
    if mode.bitdepth() == 16 {
        for i in 0..numpixels {
            let (r, g, b, a) = get_pixel_color_rgba16(inp, i, mode);
            if (r & 255) != ((r >> 8) & 255)
                || (g & 255) != ((g >> 8) & 255)
                || (b & 255) != ((b >> 8) & 255)
                || (a & 255) != ((a >> 8) & 255)
            {
                /*first and second byte differ*/
                sixteen = true;
                break;
            };
        }
    }
    if sixteen {
        profile.bits = 16;
        bits_done = true;
        numcolors_done = true;
        /*counting colors no longer useful, palette doesn't support 16-bit*/
        for i in 0..numpixels {
            let (r, g, b, a) = get_pixel_color_rgba16(inp, i, mode);
            if !colored_done && (r != g || r != b) {
                profile.colored = 1;
                colored_done = true;
            }
            if !alpha_done {
                if let Some((ref mut key_r, ref mut key_g, ref mut key_b)) =
                    profile.key
                {
                    let matchkey = r == *key_r && g == *key_g && b == *key_b;

                    if a != 65535 && (a != 0 || !matchkey) {
                        profile.alpha = true;
                        profile.key = None;
                        alpha_done = true;
                    } else if a == 65535 && matchkey {
                        profile.alpha = true;
                        profile.key = None;
                        alpha_done = true;
                    }
                } else if a == 0 && !profile.alpha {
                    profile.key = Some((r, g, b));
                }
            }
            if alpha_done && numcolors_done && colored_done && bits_done {
                break;
            };
        }
        if !profile.alpha {
            if let Some((key_r, key_g, key_b)) = profile.key {
                for i in 0..numpixels {
                    let (r, g, b, a) = get_pixel_color_rgba16(inp, i, mode);
                    if a != 0 && r == key_r && g == key_g && b == key_b {
                        profile.alpha = true;
                        profile.key = None;
                    }
                }
            }
        }
    } else {
        let mut tree = ColorTree::new();
        for i in 0..numpixels {
            let (r, g, b, a) = get_pixel_color_rgba8(inp, i, mode);
            if !bits_done && profile.bits < 8 {
                let bits = get_value_required_bits(r) as u32;
                if bits > profile.bits {
                    profile.bits = bits;
                };
            }
            bits_done = profile.bits as usize >= bpp;
            if !colored_done && (r != g || r != b) {
                profile.colored = 1;
                colored_done = true;
                if profile.bits < 8 {
                    profile.bits = 8;
                };
                /*PNG has no colored modes with less than 8-bit per channel*/
            }
            if !alpha_done {
                let matchkey = if let Some((key_r, key_g, key_b)) = profile.key
                {
                    r as u16 == key_r && g as u16 == key_g && b as u16 == key_b
                } else {
                    false
                };

                if a != 255 && (a != 0 || (profile.key.is_some() && !matchkey))
                {
                    profile.alpha = true;
                    profile.key = None;
                    alpha_done = true;
                    if profile.bits < 8 {
                        profile.bits = 8;
                    };
                /*PNG has no alphachannel modes with less than 8-bit per channel*/
                } else if a == 0 && !profile.alpha && profile.key.is_none() {
                    profile.key = Some((r as u16, g as u16, b as u16));
                } else if a == 255 && profile.key.is_some() && matchkey {
                    profile.alpha = true;
                    profile.key = None;
                    alpha_done = true;
                    if profile.bits < 8 {
                        profile.bits = 8;
                    };
                    /*PNG has no alphachannel modes with less than 8-bit per channel*/
                };
            }
            if !numcolors_done && tree.get(&(r, g, b, a)).is_none() {
                tree.insert((r, g, b, a), profile.numcolors as u16);
                if profile.numcolors < 256 {
                    profile.palette[profile.numcolors as usize] =
                        SRgba8::new(r, g, b, a);
                }
                profile.numcolors += 1;
                numcolors_done = profile.numcolors >= maxnumcolors;
            }
            if alpha_done && numcolors_done && colored_done && bits_done {
                break;
            };
        }
        if !profile.alpha {
            if let Some((key_r, key_g, key_b)) = profile.key {
                for i in 0..numpixels {
                    let (r, g, b, a) = get_pixel_color_rgba8(inp, i, mode);
                    if a != 0
                        && r as u16 == key_r
                        && g as u16 == key_g
                        && b as u16 == key_b
                    {
                        profile.alpha = true;
                        profile.key = None;
                        /*PNG has no alphachannel modes with less than 8-bit per channel*/
                        if profile.bits < 8 {
                            profile.bits = 8;
                        };
                    };
                }
            }
        }
        /*make the profile's key always 16-bit for consistency - repeat each byte twice*/
        if let Some((ref mut key_r, ref mut key_g, ref mut key_b)) = profile.key
        {
            *key_r += *key_r << 8;
            *key_g += *key_g << 8;
            *key_b += *key_b << 8;
        }
    }
    Ok(profile)
}

/*Automatically chooses color type that gives smallest amount of bits in the
output image, e.g. grey if there are only greyscale pixels, palette if there
are less than 256 colors, ...
Updates values of mode with a potentially smaller color model. mode_out should
contain the user chosen color model, but will be overwritten with the new chosen one.*/
pub(crate) fn auto_choose_color(
    image: &[u8],
    w: usize,
    h: usize,
    mode_in: &ColorMode,
) -> Result<ColorMode, Error> {
    let mut mode_out = ColorMode::new();
    let mut prof = get_color_profile(image, w as u32, h as u32, mode_in)?;

    mode_out.clear_key();
    if prof.key.is_some() && w * h <= 16 {
        prof.alpha = true;
        prof.key = None;
        /*PNG has no alphachannel modes with less than 8-bit per channel*/
        if prof.bits < 8 {
            prof.bits = 8;
        };
    }
    let n = prof.numcolors;
    let palettebits = if n <= 2 {
        1
    } else if n <= 4 {
        2
    } else if n <= 16 {
        4
    } else {
        8
    };
    let palette_ok = (n <= 256 && prof.bits <= 8)
        && (w * h >= (n * 2) as usize)
        && (prof.colored != 0 || prof.bits > palettebits);
    if palette_ok {
        let pal = &prof.palette[0..prof.numcolors as usize];
        /*remove potential earlier palette*/
        mode_out.palette_clear();
        for p in pal {
            mode_out.palette_add(*p)?;
        }
        mode_out.colortype = ColorType::Palette;
        mode_out.set_bitdepth(palettebits);
        if mode_in.colortype == ColorType::Palette
            && mode_in.palette().len() >= mode_out.palette().len()
            && mode_in.bitdepth() == mode_out.bitdepth()
        {
            /*If input should have same palette colors, keep original to preserve its order and prevent conversion*/
            mode_out = mode_in.clone();
        };
    } else {
        mode_out.set_bitdepth(prof.bits);
        mode_out.colortype = if prof.alpha {
            if prof.colored != 0 {
                ColorType::Rgba
            } else {
                ColorType::GreyAlpha
            }
        } else if prof.colored != 0 {
            ColorType::Rgb
        } else {
            ColorType::Grey
        };
        if let Some((key_r, key_g, key_b)) = prof.key {
            let mask = ((1 << mode_out.bitdepth()) - 1) as u16;
            /*profile always uses 16-bit, mask converts it*/
            mode_out.set_key(key_r & mask, key_g & mask, key_b & mask);
        };
    }
    Ok(mode_out)
}

pub(crate) fn lodepng_encode_memory(
    image: &[u8],
    w: u32,
    h: u32,
    colortype: ColorType,
    bitdepth: u32,
) -> Result<Vec<u8>, Error> {
    let mut state = State::new();
    state.info_raw.colortype = colortype;
    state.info_raw.set_bitdepth(bitdepth);
    state.info_png.color.colortype = colortype;
    state.info_png.color.set_bitdepth(bitdepth);
    lodepng_encode(image, w, h, &mut state)
}

impl ColorProfile {
    pub(crate) fn new() -> Self {
        Self {
            colored: 0,
            key: None,
            alpha: false,
            numcolors: 0,
            bits: 1,
            palette: [SRgba8::new(0, 0, 0, 0); 256],
        }
    }
}

/*Returns how many bits needed to represent given value (max 8 bit)*/
pub(super) fn get_value_required_bits(value: u8) -> u8 {
    match value {
        0 | 255 => 1,
        x if x % 17 == 0 => {
            /*The scaling of 2-bit and 4-bit values uses multiples of 85 and 17*/
            if value % 85 == 0 {
                2
            } else {
                4
            }
        }
        _ => 8,
    }
}
