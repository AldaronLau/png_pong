//! Deflate - Huffman

use pix::Alpha;
use pix::Ch8;

use crate::chunk::ITextChunk;
use crate::chunk::TextChunk;

use super::*;

pub(crate) fn string_copy(inp: &[u8]) -> String {
    String::from_utf8_lossy(inp).to_string()
}

#[inline]
pub(super) fn lodepng_read32bit_int(buffer: &[u8]) -> u32 {
    ((buffer[0] as u32) << 24)
        | ((buffer[1] as u32) << 16)
        | ((buffer[2] as u32) << 8)
        | buffer[3] as u32
}

#[inline(always)]
pub(super) fn lodepng_set32bit_int(buffer: &mut [u8], value: u32) {
    buffer[0] = ((value >> 24) & 255) as u8;
    buffer[1] = ((value >> 16) & 255) as u8;
    buffer[2] = ((value >> 8) & 255) as u8;
    buffer[3] = ((value) & 255) as u8;
}

#[inline(always)]
fn add32bit_int(buffer: &mut Vec<u8>, value: u32) {
    buffer.push(((value >> 24) & 255) as u8);
    buffer.push(((value >> 16) & 255) as u8);
    buffer.push(((value >> 8) & 255) as u8);
    buffer.push(((value) & 255) as u8);
}

#[inline]
pub(super) fn lodepng_add32bit_int(buffer: &mut Vec<u8>, value: u32) {
    add32bit_int(buffer, value);
}

pub(crate) fn text_copy(dest: &mut Info, source: &Info) -> Result<(), Error> {
    dest.text.clear();
    for ntext in source.text_keys_cstr() {
        dest.push_text(ntext.key.as_bytes(), ntext.val.as_bytes());
    }
    Ok(())
}

impl Info {
    pub(crate) fn push_itext(
        &mut self,
        key: &[u8],
        langtag: &[u8],
        transkey: &[u8],
        str: &[u8],
    ) -> Result<(), Error> {
        self.itext.push(ITextChunk {
            key: string_copy(key),
            langtag: string_copy(langtag),
            transkey: string_copy(transkey),
            val: string_copy(str),
        });

        Ok(())
    }

    pub(crate) fn push_text(&mut self, k: &[u8], v: &[u8]) {
        self.text.push(TextChunk {
            key: string_copy(k),
            val: string_copy(v),
        });
    }

    pub(super) fn push_unknown_chunk(
        &mut self,
        critical_pos: ChunkPosition,
        chunk: &[u8],
    ) -> Result<(), Error> {
        let set = critical_pos as usize;

        chunk_append(&mut self.unknown_chunks_data[set], chunk);

        Ok(())
    }

    #[inline]
    pub(super) fn unknown_chunks_data(
        &self,
        critical_pos: ChunkPosition,
    ) -> Option<&[u8]> {
        let set = critical_pos as usize;

        if self.unknown_chunks_data[set].is_empty() {
            return None;
        }

        Some(self.unknown_chunks_data[set].as_slice())
    }
}

pub(crate) fn itext_copy(dest: &mut Info, source: &Info) -> Result<(), Error> {
    dest.itext.clear();
    for itext in source.itext_keys() {
        dest.push_itext(
            itext.key.as_bytes(),
            itext.langtag.as_bytes(),
            itext.transkey.as_bytes(),
            itext.val.as_bytes(),
        )?;
    }
    Ok(())
}

fn add_color_bits(out: &mut [u8], index: usize, bits: u32, mut inp: u32) {
    let m = match bits {
        1 => 7,
        2 => 3,
        _ => 1,
    };
    /*p = the partial index in the byte, e.g. with 4 palettebits it is 0 for first half or 1 for second half*/
    let p = index & m; /*filter out any other bits of the input value*/
    inp &= (1 << bits) - 1;
    inp <<= bits * (m - p) as u32;
    if p == 0 {
        out[index * bits as usize / 8] = inp as u8;
    } else {
        out[index * bits as usize / 8] |= inp as u8;
    }
}

pub type ColorTree = HashMap<(u8, u8, u8, u8), u16>;

#[inline(always)]
pub(super) fn rgba8_to_pixel(
    out: &mut [u8],
    i: usize,
    mode: &ColorMode,
    tree: &mut ColorTree,
    /*for palette*/ rgba: [u8; 4],
) -> Result<(), Error> {
    match mode.colortype {
        ColorType::Grey => {
            let grey = rgba[0]; /*((unsigned short)r + g + b) / 3*/
            if mode.bitdepth() == 8 {
                out[i] = grey; /*take the most significant bits of grey*/
            } else if mode.bitdepth() == 16 {
                out[i * 2 + 0] = {
                    out[i * 2 + 1] = grey; /*color not in palette*/
                    out[i * 2 + 1]
                }; /*((unsigned short)r + g + b) / 3*/
            } else {
                let grey = (grey >> (8 - mode.bitdepth()))
                    & ((1 << mode.bitdepth()) - 1); /*no error*/
                add_color_bits(out, i, mode.bitdepth(), grey.into());
            };
        }
        ColorType::Rgb => {
            if mode.bitdepth() == 8 {
                out[i * 3 + 0] = rgba[0];
                out[i * 3 + 1] = rgba[1];
                out[i * 3 + 2] = rgba[2];
            } else {
                out[i * 6 + 0] = rgba[0];
                out[i * 6 + 1] = rgba[0];
                out[i * 6 + 2] = rgba[1];
                out[i * 6 + 3] = rgba[1];
                out[i * 6 + 4] = rgba[2];
                out[i * 6 + 5] = rgba[2];
            }
        }
        ColorType::Palette => {
            let [red, green, blue, alpha] = rgba;
            let index =
                *tree.get(&(red, green, blue, alpha)).ok_or(Error(82))?;
            if mode.bitdepth() == 8 {
                out[i] = index as u8;
            } else {
                add_color_bits(out, i, mode.bitdepth(), u32::from(index));
            };
        }
        ColorType::GreyAlpha => {
            let grey = rgba[0];
            let alpha = rgba[3];
            if mode.bitdepth() == 8 {
                out[i * 2 + 0] = grey;
                out[i * 2 + 1] = alpha;
            } else if mode.bitdepth() == 16 {
                out[i * 4 + 0] = grey;
                out[i * 4 + 1] = grey;
                out[i * 4 + 2] = alpha;
                out[i * 4 + 3] = alpha;
            }
        }
        ColorType::Rgba => {
            if mode.bitdepth() == 8 {
                out[i * 4 + 0] = rgba[0];
                out[i * 4 + 1] = rgba[1];
                out[i * 4 + 2] = rgba[2];
                out[i * 4 + 3] = rgba[3];
            } else {
                out[i * 8 + 0] = rgba[0];
                out[i * 8 + 1] = rgba[0];
                out[i * 8 + 2] = rgba[1];
                out[i * 8 + 3] = rgba[1];
                out[i * 8 + 4] = rgba[2];
                out[i * 8 + 5] = rgba[2];
                out[i * 8 + 6] = rgba[3];
                out[i * 8 + 7] = rgba[3];
            }
        }
        ColorType::Bgra | ColorType::Bgr | ColorType::Bgrx => {
            return Err(Error(31));
        }
    };
    Ok(())
}

/*put a pixel, given its RGBA16 color, into image of any color 16-bitdepth type*/
#[inline(always)]
pub(super) fn rgba16_to_pixel(
    out: &mut [u8],
    i: usize,
    mode: &ColorMode,
    r: u16,
    g: u16,
    b: u16,
    a: u16,
) {
    match mode.colortype {
        ColorType::Grey => {
            let grey = r;
            out[i * 2 + 0] = (grey >> 8) as u8;
            out[i * 2 + 1] = grey as u8;
        }
        ColorType::Rgb => {
            out[i * 6 + 0] = (r >> 8) as u8;
            out[i * 6 + 1] = r as u8;
            out[i * 6 + 2] = (g >> 8) as u8;
            out[i * 6 + 3] = g as u8;
            out[i * 6 + 4] = (b >> 8) as u8;
            out[i * 6 + 5] = b as u8;
        }
        ColorType::GreyAlpha => {
            let grey = r;
            out[i * 4 + 0] = (grey >> 8) as u8;
            out[i * 4 + 1] = grey as u8;
            out[i * 4 + 2] = (a >> 8) as u8;
            out[i * 4 + 3] = a as u8;
        }
        ColorType::Rgba => {
            out[i * 8 + 0] = (r >> 8) as u8;
            out[i * 8 + 1] = r as u8;
            out[i * 8 + 2] = (g >> 8) as u8;
            out[i * 8 + 3] = g as u8;
            out[i * 8 + 4] = (b >> 8) as u8;
            out[i * 8 + 5] = b as u8;
            out[i * 8 + 6] = (a >> 8) as u8;
            out[i * 8 + 7] = a as u8;
        }
        ColorType::Bgr
        | ColorType::Bgra
        | ColorType::Bgrx
        | ColorType::Palette => unreachable!(),
    };
}

/*Get RGBA8 color of pixel with index i (y * width + x) from the raw image with given color type.*/
pub(super) fn get_pixel_color_rgba8(
    inp: &[u8],
    i: usize,
    mode: &ColorMode,
) -> (u8, u8, u8, u8) {
    match mode.colortype {
        ColorType::Grey => {
            if mode.bitdepth() == 8 {
                let t = inp[i];
                let a = if mode.key()
                    == Some((u16::from(t), u16::from(t), u16::from(t)))
                {
                    0
                } else {
                    255
                };
                (t, t, t, a)
            } else if mode.bitdepth() == 16 {
                let t = inp[i * 2 + 0];
                let g = 256 * inp[i * 2 + 0] as u16 + inp[i * 2 + 1] as u16;
                let a = if mode.key() == Some((g, g, g)) {
                    0
                } else {
                    255
                };
                (t, t, t, a)
            } else {
                let highest = (1 << mode.bitdepth()) - 1;
                /*highest possible value for this bit depth*/
                let mut j = i as usize * mode.bitdepth() as usize;
                let value = read_bits_from_reversed_stream(
                    &mut j,
                    inp,
                    mode.bitdepth() as usize,
                );
                let t = ((value * 255) / highest) as u8;
                let a = if mode.key() == Some((t as u16, t as u16, t as u16)) {
                    0
                } else {
                    255
                };
                (t, t, t, a)
            }
        }
        ColorType::Rgb => {
            if mode.bitdepth() == 8 {
                let r = inp[i * 3 + 0];
                let g = inp[i * 3 + 1];
                let b = inp[i * 3 + 2];
                let a = if mode.key()
                    == Some((u16::from(r), u16::from(g), u16::from(b)))
                {
                    0
                } else {
                    255
                };
                (r, g, b, a)
            } else {
                (
                    inp[i * 6 + 0],
                    inp[i * 6 + 2],
                    inp[i * 6 + 4],
                    if mode.key()
                        == Some((
                            256 * inp[i * 6 + 0] as u16 + inp[i * 6 + 1] as u16,
                            256 * inp[i * 6 + 2] as u16 + inp[i * 6 + 3] as u16,
                            256 * inp[i * 6 + 4] as u16 + inp[i * 6 + 5] as u16,
                        ))
                    {
                        0
                    } else {
                        255
                    },
                )
            }
        }
        ColorType::Palette => {
            let index = if mode.bitdepth() == 8 {
                inp[i] as usize
            } else {
                let mut j = i as usize * mode.bitdepth() as usize;
                read_bits_from_reversed_stream(
                    &mut j,
                    inp,
                    mode.bitdepth() as usize,
                ) as usize
            };
            let pal = mode.palette();
            if index >= pal.len() {
                /*This is an error according to the PNG spec, but common PNG decoders make it black instead.
                Done here too, slightly faster due to no error handling needed.*/
                (0, 0, 0, 255)
            } else {
                let p = pal[index];
                (
                    p.red().into(),
                    p.green().into(),
                    p.blue().into(),
                    p.alpha().value().into(),
                )
            }
        }
        ColorType::GreyAlpha => {
            if mode.bitdepth() == 8 {
                let t = inp[i * 2 + 0];
                (t, t, t, inp[i * 2 + 1])
            } else {
                let t = inp[i * 4 + 0];
                (t, t, t, inp[i * 4 + 2])
            }
        }
        ColorType::Rgba => {
            if mode.bitdepth() == 8 {
                (
                    inp[i * 4 + 0],
                    inp[i * 4 + 1],
                    inp[i * 4 + 2],
                    inp[i * 4 + 3],
                )
            } else {
                (
                    inp[i * 8 + 0],
                    inp[i * 8 + 2],
                    inp[i * 8 + 4],
                    inp[i * 8 + 6],
                )
            }
        }
        ColorType::Bgra => (
            inp[i * 4 + 2],
            inp[i * 4 + 1],
            inp[i * 4 + 0],
            inp[i * 4 + 3],
        ),
        ColorType::Bgr => {
            let b = inp[i * 3 + 0];
            let g = inp[i * 3 + 1];
            let r = inp[i * 3 + 2];
            let a = if mode.key()
                == Some((u16::from(r), u16::from(g), u16::from(b)))
            {
                0
            } else {
                255
            };
            (r, g, b, a)
        }
        ColorType::Bgrx => {
            let b = inp[i * 4 + 0];
            let g = inp[i * 4 + 1];
            let r = inp[i * 4 + 2];
            let a = if mode.key()
                == Some((u16::from(r), u16::from(g), u16::from(b)))
            {
                0
            } else {
                255
            };
            (r, g, b, a)
        }
    }
}
/*Similar to get_pixel_color_rgba8, but with all the for loops inside of the color
mode test cases, optimized to convert the colors much faster, when converting
to RGBA or RGB with 8 bit per cannel. buffer must be RGBA or RGB output with
enough memory, if has_alpha is true the output is RGBA. mode has the color mode
of the input buffer.*/
pub(super) fn get_pixel_colors_rgba8(
    buffer: &mut [u8],
    numpixels: usize,
    has_alpha: bool,
    inp: &[u8],
    mode: &ColorMode,
) {
    let num_channels = if has_alpha { 4 } else { 3 };
    match mode.colortype {
        ColorType::Grey => {
            if mode.bitdepth() == 8 {
                for (i, buffer) in
                    buffer.chunks_mut(num_channels).take(numpixels).enumerate()
                {
                    buffer[0] = inp[i];
                    buffer[1] = inp[i];
                    buffer[2] = inp[i];
                    if has_alpha {
                        let a = inp[i] as u16;
                        buffer[3] = if mode.key() == Some((a, a, a)) {
                            0
                        } else {
                            255
                        };
                    }
                }
            } else if mode.bitdepth() == 16 {
                for (i, buffer) in
                    buffer.chunks_mut(num_channels).take(numpixels).enumerate()
                {
                    buffer[0] = inp[i * 2];
                    buffer[1] = inp[i * 2];
                    buffer[2] = inp[i * 2];
                    if has_alpha {
                        let a =
                            256 * inp[i * 2 + 0] as u16 + inp[i * 2 + 1] as u16;
                        buffer[3] = if mode.key() == Some((a, a, a)) {
                            0
                        } else {
                            255
                        };
                    };
                }
            } else {
                let highest = (1 << mode.bitdepth()) - 1;
                /*highest possible value for this bit depth*/
                let mut j = 0;
                for buffer in buffer.chunks_mut(num_channels).take(numpixels) {
                    let value = read_bits_from_reversed_stream(
                        &mut j,
                        inp,
                        mode.bitdepth() as usize,
                    );
                    buffer[0] = ((value * 255) / highest) as u8;
                    buffer[1] = ((value * 255) / highest) as u8;
                    buffer[2] = ((value * 255) / highest) as u8;
                    if has_alpha {
                        let a = value as u16;
                        buffer[3] = if mode.key() == Some((a, a, a)) {
                            0
                        } else {
                            255
                        };
                    };
                }
            };
        }
        ColorType::Rgb => {
            if mode.bitdepth() == 8 {
                for (i, buffer) in
                    buffer.chunks_mut(num_channels).take(numpixels).enumerate()
                {
                    buffer[0] = inp[i * 3 + 0];
                    buffer[1] = inp[i * 3 + 1];
                    buffer[2] = inp[i * 3 + 2];
                    if has_alpha {
                        buffer[3] = if mode.key()
                            == Some((
                                buffer[0] as u16,
                                buffer[1] as u16,
                                buffer[2] as u16,
                            )) {
                            0
                        } else {
                            255
                        };
                    };
                }
            } else {
                for (i, buffer) in
                    buffer.chunks_mut(num_channels).take(numpixels).enumerate()
                {
                    buffer[0] = inp[i * 6 + 0];
                    buffer[1] = inp[i * 6 + 2];
                    buffer[2] = inp[i * 6 + 4];
                    if has_alpha {
                        let r =
                            256 * inp[i * 6 + 0] as u16 + inp[i * 6 + 1] as u16;
                        let g =
                            256 * inp[i * 6 + 2] as u16 + inp[i * 6 + 3] as u16;
                        let b =
                            256 * inp[i * 6 + 4] as u16 + inp[i * 6 + 5] as u16;
                        buffer[3] = if mode.key() == Some((r, g, b)) {
                            0
                        } else {
                            255
                        };
                    };
                }
            };
        }
        ColorType::Palette => {
            let mut j = 0;
            for (i, buffer) in
                buffer.chunks_mut(num_channels).take(numpixels).enumerate()
            {
                let index = if mode.bitdepth() == 8 {
                    inp[i] as usize
                } else {
                    read_bits_from_reversed_stream(
                        &mut j,
                        inp,
                        mode.bitdepth() as usize,
                    ) as usize
                };
                let pal = mode.palette();
                if index >= pal.len() {
                    /*This is an error according to the PNG spec, but most PNG decoders make it black instead.
                    Done here too, slightly faster due to no error handling needed.*/
                    buffer[0] = 0;
                    buffer[1] = 0;
                    buffer[2] = 0;
                    if has_alpha {
                        buffer[3] = 255u8;
                    }
                } else {
                    let p = pal[index as usize];
                    buffer[0] = p.red().into();
                    buffer[1] = p.green().into();
                    buffer[2] = p.blue().into();
                    if has_alpha {
                        buffer[3] = p.alpha().value().into();
                    }
                };
            }
        }
        ColorType::GreyAlpha => {
            if mode.bitdepth() == 8 {
                for (i, buffer) in
                    buffer.chunks_mut(num_channels).take(numpixels).enumerate()
                {
                    buffer[0] = inp[i * 2 + 0];
                    buffer[1] = inp[i * 2 + 0];
                    buffer[2] = inp[i * 2 + 0];
                    if has_alpha {
                        buffer[3] = inp[i * 2 + 1];
                    };
                }
            } else {
                for (i, buffer) in
                    buffer.chunks_mut(num_channels).take(numpixels).enumerate()
                {
                    buffer[0] = inp[i * 4 + 0];
                    buffer[1] = inp[i * 4 + 0];
                    buffer[2] = inp[i * 4 + 0];
                    if has_alpha {
                        buffer[3] = inp[i * 4 + 2];
                    };
                }
            }
        }
        ColorType::Rgba => {
            if mode.bitdepth() == 8 {
                for (i, buffer) in
                    buffer.chunks_mut(num_channels).take(numpixels).enumerate()
                {
                    buffer[0] = inp[i * 4 + 0];
                    buffer[1] = inp[i * 4 + 1];
                    buffer[2] = inp[i * 4 + 2];
                    if has_alpha {
                        buffer[3] = inp[i * 4 + 3];
                    }
                }
            } else {
                for (i, buffer) in
                    buffer.chunks_mut(num_channels).take(numpixels).enumerate()
                {
                    buffer[0] = inp[i * 8 + 0];
                    buffer[1] = inp[i * 8 + 2];
                    buffer[2] = inp[i * 8 + 4];
                    if has_alpha {
                        buffer[3] = inp[i * 8 + 6];
                    }
                }
            }
        }
        ColorType::Bgr => {
            for (i, buffer) in
                buffer.chunks_mut(num_channels).take(numpixels).enumerate()
            {
                buffer[0] = inp[i * 3 + 2];
                buffer[1] = inp[i * 3 + 1];
                buffer[2] = inp[i * 3 + 0];
                if has_alpha {
                    buffer[3] = if mode.key()
                        == Some((
                            buffer[0] as u16,
                            buffer[1] as u16,
                            buffer[2] as u16,
                        )) {
                        0
                    } else {
                        255
                    };
                };
            }
        }
        ColorType::Bgrx => {
            for (i, buffer) in
                buffer.chunks_mut(num_channels).take(numpixels).enumerate()
            {
                buffer[0] = inp[i * 4 + 2];
                buffer[1] = inp[i * 4 + 1];
                buffer[2] = inp[i * 4 + 0];
                if has_alpha {
                    buffer[3] = if mode.key()
                        == Some((
                            buffer[0] as u16,
                            buffer[1] as u16,
                            buffer[2] as u16,
                        )) {
                        0
                    } else {
                        255
                    };
                };
            }
        }
        ColorType::Bgra => {
            for (i, buffer) in
                buffer.chunks_mut(num_channels).take(numpixels).enumerate()
            {
                buffer[0] = inp[i * 4 + 2];
                buffer[1] = inp[i * 4 + 1];
                buffer[2] = inp[i * 4 + 0];
                if has_alpha {
                    buffer[3] = inp[i * 4 + 3];
                }
            }
        }
    };
}
/*Get RGBA16 color of pixel with index i (y * width + x) from the raw image with
given color type, but the given color type must be 16-bit itself.*/
#[inline(always)]
pub(super) fn get_pixel_color_rgba16(
    inp: &[u8],
    i: usize,
    mode: &ColorMode,
) -> (u16, u16, u16, u16) {
    match mode.colortype {
        ColorType::Grey => {
            let t = 256 * inp[i * 2 + 0] as u16 + inp[i * 2 + 1] as u16;
            (
                t,
                t,
                t,
                if mode.key() == Some((t, t, t)) {
                    0
                } else {
                    0xffff
                },
            )
        }
        ColorType::Rgb => {
            let r = 256 * inp[i * 6 + 0] as u16 + inp[i * 6 + 1] as u16;
            let g = 256 * inp[i * 6 + 2] as u16 + inp[i * 6 + 3] as u16;
            let b = 256 * inp[i * 6 + 4] as u16 + inp[i * 6 + 5] as u16;
            let a = if mode.key() == Some((r, g, b)) {
                0
            } else {
                0xffff
            };
            (r, g, b, a)
        }
        ColorType::GreyAlpha => {
            let t = 256 * inp[i * 4 + 0] as u16 + inp[i * 4 + 1] as u16;
            let a = 256 * inp[i * 4 + 2] as u16 + inp[i * 4 + 3] as u16;
            (t, t, t, a)
        }
        ColorType::Rgba => (
            256 * inp[i * 8 + 0] as u16 + inp[i * 8 + 1] as u16,
            256 * inp[i * 8 + 2] as u16 + inp[i * 8 + 3] as u16,
            256 * inp[i * 8 + 4] as u16 + inp[i * 8 + 5] as u16,
            256 * inp[i * 8 + 6] as u16 + inp[i * 8 + 7] as u16,
        ),
        ColorType::Bgr
        | ColorType::Bgra
        | ColorType::Bgrx
        | ColorType::Palette => unreachable!(),
    }
}

#[inline(always)]
fn read_bits_from_reversed_stream(
    bitpointer: &mut usize,
    bitstream: &[u8],
    nbits: usize,
) -> u32 {
    let mut result = 0;
    for _ in 0..nbits {
        result <<= 1;
        result |= read_bit_from_reversed_stream(bitpointer, bitstream) as u32;
    }
    result
}

pub(super) fn read_chunk_plte(
    color: &mut ColorMode,
    data: &[u8],
) -> Result<(), Error> {
    color.palette_clear();
    for c in data.chunks(3).take(data.len() / 3) {
        color.palette_add(Rgba8::with_alpha(
            Ch8::new(c[0]),
            Ch8::new(c[1]),
            Ch8::new(c[2]),
            Ch8::new(255),
        ))?;
    }
    Ok(())
}

pub(super) fn read_chunk_trns(
    color: &mut ColorMode,
    data: &[u8],
) -> Result<(), Error> {
    if color.colortype == ColorType::Palette {
        let pal = color.palette_mut();
        if data.len() > pal.len() {
            return Err(Error(38));
        }
        for (i, &d) in data.iter().enumerate() {
            pal[i] = Rgba8::with_alpha(
                pal[i].red(),
                pal[i].green(),
                pal[i].blue(),
                Ch8::new(d),
            );
        }
    } else if color.colortype == ColorType::Grey {
        if data.len() != 2 {
            return Err(Error(30));
        }
        let t = 256 * data[0] as u16 + data[1] as u16;
        color.set_key(t, t, t);
    } else if color.colortype == ColorType::Rgb {
        if data.len() != 6 {
            return Err(Error(41));
        }
        color.set_key(
            256 * data[0] as u16 + data[1] as u16,
            256 * data[2] as u16 + data[3] as u16,
            256 * data[4] as u16 + data[5] as u16,
        );
    } else {
        return Err(Error(42));
    }
    Ok(())
}

/*background color chunk (bKGD)*/
pub(super) fn read_chunk_bkgd(
    info: &mut Info,
    data: &[u8],
) -> Result<(), Error> {
    let chunk_length = data.len();
    if info.color.colortype == ColorType::Palette {
        /*error: this chunk must be 1 byte for indexed color image*/
        if chunk_length != 1 {
            return Err(Error(43)); /*error: this chunk must be 2 bytes for greyscale image*/
        } /*error: this chunk must be 6 bytes for greyscale image*/
        info.background_defined = 1; /* OK */
        info.background_r = {
            info.background_g = {
                info.background_b = data[0] as u32;
                info.background_b
            };
            info.background_g
        };
    } else if info.color.colortype == ColorType::Grey
        || info.color.colortype == ColorType::GreyAlpha
    {
        if chunk_length != 2 {
            return Err(Error(44));
        }
        info.background_defined = 1;
        info.background_r = {
            info.background_g = {
                info.background_b = 256 * data[0] as u32 + data[1] as u32;
                info.background_b
            };
            info.background_g
        };
    } else if info.color.colortype == ColorType::Rgb
        || info.color.colortype == ColorType::Rgba
    {
        if chunk_length != 6 {
            return Err(Error(45));
        }
        info.background_defined = 1;
        info.background_r = 256 * data[0] as u32 + data[1] as u32;
        info.background_g = 256 * data[2] as u32 + data[3] as u32;
        info.background_b = 256 * data[4] as u32 + data[5] as u32;
    }
    Ok(())
}

/*text chunk (tEXt)*/
pub(super) fn read_chunk_text(
    info: &mut Info,
    data: &[u8],
) -> Result<(), Error> {
    let (keyword, str) = split_at_nul(data);
    if keyword.is_empty() || keyword.len() > 79 {
        return Err(Error(89));
    }
    /*even though it's not allowed by the standard, no error is thrown if
    there's no null termination char, if the text is empty*/
    Ok(info.push_text(keyword, str))
}

/*compressed text chunk (zTXt)*/
pub(super) fn read_chunk_ztxt(
    info: &mut Info,
    zlibsettings: &DecompressSettings,
    data: &[u8],
) -> Result<(), Error> {
    let mut length = 0;
    while length < data.len() && data[length] != 0 {
        length += 1
    }
    if length + 2 >= data.len() {
        return Err(Error(75));
    }
    if length < 1 || length > 79 {
        return Err(Error(89));
    }
    let key = &data[0..length];
    if data[length + 1] != 0 {
        return Err(Error(72));
    }
    /*the 0 byte indicating compression must be 0*/
    let string2_begin = length + 2; /*no null termination, corrupt?*/
    if string2_begin > data.len() {
        return Err(Error(75)); /*will fail if zlib error, e.g. if length is too small*/
    }
    let inl = &data[string2_begin..];
    let decoded = zlib_decompress(inl, zlibsettings)?;
    info.push_text(key, &decoded);
    Ok(())
}

fn split_at_nul(data: &[u8]) -> (&[u8], &[u8]) {
    let mut part = data.splitn(2, |&b| b == 0);
    (part.next().unwrap(), part.next().unwrap_or(&data[0..0]))
}

/*international text chunk (iTXt)*/
pub(super) fn read_chunk_itxt(
    info: &mut Info,
    zlibsettings: &DecompressSettings,
    data: &[u8],
) -> Result<(), Error> {
    /*Quick check if the chunk length isn't too small. Even without check
    it'd still fail with other error checks below if it's too short. This just gives a different error code.*/
    if data.len() < 5 {
        /*iTXt chunk too short*/
        return Err(Error(30));
    }

    let (key, data) = split_at_nul(data);
    if key.is_empty() || key.len() > 79 {
        return Err(Error(89));
    }
    if data.len() < 2 {
        return Err(Error(75));
    }
    let compressed_flag = data[0];
    if data[1] != 0 {
        return Err(Error(72));
    }
    let (langtag, data) = split_at_nul(&data[2..]);
    let (transkey, data) = split_at_nul(data);

    let decoded;
    let rest = if compressed_flag != 0 {
        decoded = zlib_decompress(data, zlibsettings)?;
        &decoded[..]
    } else {
        data
    };
    info.push_itext(key, langtag, transkey, rest)?;
    Ok(())
}

pub(super) fn read_chunk_time(
    info: &mut Info,
    data: &[u8],
) -> Result<(), Error> {
    let chunk_length = data.len();
    if chunk_length != 7 {
        return Err(Error(73));
    }
    info.time_defined = 1;
    info.time.year = 256 * data[0] as u32 + data[1] as u32;
    info.time.month = data[2] as u32;
    info.time.day = data[3] as u32;
    info.time.hour = data[4] as u32;
    info.time.minute = data[5] as u32;
    info.time.second = data[6] as u32;
    Ok(())
}

pub(super) fn read_chunk_phys(
    info: &mut Info,
    data: &[u8],
) -> Result<(), Error> {
    let chunk_length = data.len();
    if chunk_length != 9 {
        return Err(Error(74));
    }
    info.phys_defined = 1;
    info.phys_x = 16777216 * data[0] as u32
        + 65536 * data[1] as u32
        + 256 * data[2] as u32
        + data[3] as u32;
    info.phys_y = 16777216 * data[4] as u32
        + 65536 * data[5] as u32
        + 256 * data[6] as u32
        + data[7] as u32;
    info.phys_unit = data[8] as u32;
    Ok(())
}

pub(super) fn add_chunk_idat(
    out: &mut Vec<u8>,
    data: &[u8],
    zlibsettings: &CompressSettings,
) -> Result<(), Error> {
    let zlib = zlib_compress(data, zlibsettings)?;
    add_chunk(out, b"IDAT", &zlib)?;
    Ok(())
}

pub(super) fn add_chunk_iend(out: &mut Vec<u8>) -> Result<(), Error> {
    add_chunk(out, b"IEND", &[])
}

pub(super) fn add_chunk_text(
    out: &mut Vec<u8>,
    keyword: &str,
    textstring: &str,
) -> Result<(), Error> {
    if keyword.as_bytes().is_empty() || keyword.as_bytes().len() > 79 {
        return Err(Error(89));
    }
    let mut text = Vec::from(keyword.as_bytes());
    text.push(0u8);
    text.extend_from_slice(textstring.as_bytes());
    add_chunk(out, b"tEXt", &text)
}

pub(super) fn add_chunk_ztxt(
    out: &mut Vec<u8>,
    keyword: &str,
    textstring: &str,
    zlibsettings: &CompressSettings,
) -> Result<(), Error> {
    if keyword.as_bytes().is_empty() || keyword.as_bytes().len() > 79 {
        return Err(Error(89));
    }
    let mut data = Vec::from(keyword.as_bytes());
    data.push(0u8); // TODO: 2x?
    let textstring = textstring.as_bytes();
    let v = zlib_compress(textstring, zlibsettings)?;
    data.extend_from_slice(&v);
    add_chunk(out, b"zTXt", &data)?;
    Ok(())
}

pub(super) fn add_chunk_itxt(
    out: &mut Vec<u8>,
    compressed: bool,
    keyword: &str,
    langtag: &str,
    transkey: &str,
    textstring: &str,
    zlibsettings: &CompressSettings,
) -> Result<(), Error> {
    let k_len = keyword.len();
    if k_len < 1 || k_len > 79 {
        return Err(Error(89));
    }
    let mut data = Vec::new();
    data.extend_from_slice(keyword.as_bytes());
    data.push(0);
    data.push(compressed as u8);
    data.push(0);
    data.extend_from_slice(langtag.as_bytes());
    data.push(0);
    data.extend_from_slice(transkey.as_bytes());
    data.push(0);
    if compressed {
        let compressed_data =
            zlib_compress(textstring.as_bytes(), zlibsettings)?;
        data.extend_from_slice(&compressed_data);
    } else {
        data.extend_from_slice(textstring.as_bytes());
    }
    add_chunk(out, b"iTXt", &data)
}

pub(super) fn add_chunk_bkgd(
    out: &mut Vec<u8>,
    info: &Info,
) -> Result<(), Error> {
    let mut bkgd = Vec::new();
    if info.color.colortype == ColorType::Grey
        || info.color.colortype == ColorType::GreyAlpha
    {
        bkgd.push((info.background_r >> 8) as u8);
        bkgd.push((info.background_r & 255) as u8);
    } else if info.color.colortype == ColorType::Rgb
        || info.color.colortype == ColorType::Rgba
    {
        bkgd.push((info.background_r >> 8) as u8);
        bkgd.push((info.background_r & 255) as u8);
        bkgd.push((info.background_g >> 8) as u8);
        bkgd.push((info.background_g & 255) as u8);
        bkgd.push((info.background_b >> 8) as u8);
        bkgd.push((info.background_b & 255) as u8);
    } else if info.color.colortype == ColorType::Palette {
        bkgd.push((info.background_r & 255) as u8);
    }
    add_chunk(out, b"bKGD", &bkgd)
}

pub(super) fn add_chunk_ihdr(
    out: &mut Vec<u8>,
    w: usize,
    h: usize,
    colortype: ColorType,
    bitdepth: usize,
    interlace_method: u8,
) -> Result<(), Error> {
    let mut header = Vec::new();
    add32bit_int(&mut header, w as u32);
    add32bit_int(&mut header, h as u32);
    header.push(bitdepth as u8);
    header.push(colortype as u8);
    header.push(0u8);
    header.push(0u8);
    header.push(interlace_method);
    add_chunk(out, b"IHDR", &header)
}

pub(super) fn add_chunk_trns(
    out: &mut Vec<u8>,
    info: &ColorMode,
) -> Result<(), Error> {
    let mut trns = Vec::new();
    if info.colortype == ColorType::Palette {
        let palette = info.palette();
        let mut amount = palette.len();
        /*the tail of palette values that all have 255 as alpha, does not have to be encoded*/
        let mut i = palette.len();
        while i != 0 {
            let byte: u8 = palette[i - 1].alpha().value().into();
            if byte == 255 {
                amount -= 1;
            } else {
                break;
            };
            i -= 1;
        }
        for p in &palette[0..amount] {
            trns.push(p.alpha().value().into());
        }
    } else if info.colortype == ColorType::Grey {
        if let Some((r, _, _)) = info.key() {
            trns.push((r >> 8) as u8);
            trns.push((r & 255) as u8);
        };
    } else if info.colortype == ColorType::Rgb {
        if let Some((r, g, b)) = info.key() {
            trns.push((r >> 8) as u8);
            trns.push((r & 255) as u8);
            trns.push((g >> 8) as u8);
            trns.push((g & 255) as u8);
            trns.push((b >> 8) as u8);
            trns.push((b & 255) as u8);
        };
    }
    add_chunk(out, b"tRNS", &trns)
}

pub(super) fn add_chunk_plte(
    out: &mut Vec<u8>,
    info: &ColorMode,
) -> Result<(), Error> {
    let mut plte = Vec::new();
    for p in info.palette() {
        plte.push(p.red().into());
        plte.push(p.green().into());
        plte.push(p.blue().into());
    }
    add_chunk(out, b"PLTE", &plte)
}

pub(super) fn add_chunk_time(
    out: &mut Vec<u8>,
    time: &Time,
) -> Result<(), Error> {
    let data = [
        (time.year >> 8) as u8,
        (time.year & 255) as u8,
        time.month as u8,
        time.day as u8,
        time.hour as u8,
        time.minute as u8,
        time.second as u8,
    ];
    add_chunk(out, b"tIME", &data)
}

pub(super) fn add_chunk_phys(
    out: &mut Vec<u8>,
    info: &Info,
) -> Result<(), Error> {
    let mut data = Vec::new();
    add32bit_int(&mut data, info.phys_x);
    add32bit_int(&mut data, info.phys_y);
    data.push(info.phys_unit as u8);
    add_chunk(out, b"pHYs", &data)
}

/*chunk_name must be string of 4 characters*/
pub(crate) fn add_chunk(
    out: &mut Vec<u8>,
    type_: &[u8; 4],
    data: &[u8],
) -> Result<(), Error> {
    let length = data.len() as usize;
    if length > (1 << 31) {
        return Err(Error(77));
    }
    let previous_length = out.len();
    out.reserve(length + 12);
    /*1: length*/
    lodepng_add32bit_int(out, length as u32);
    /*2: chunk name (4 letters)*/
    out.extend_from_slice(&type_[..]);
    /*3: the data*/
    out.extend_from_slice(data);
    /*4: CRC (of the chunkname characters and the data)*/
    lodepng_add32bit_int(out, 0);
    lodepng_chunk_generate_crc(&mut out[previous_length..]);
    Ok(())
}

/*shared values used by multiple Adam7 related functions*/
pub const ADAM7_IX: [u32; 7] = [0, 4, 0, 2, 0, 1, 0];
/*x start values*/
pub const ADAM7_IY: [u32; 7] = [0, 0, 4, 0, 2, 0, 1];
/*y start values*/
pub const ADAM7_DX: [u32; 7] = [8, 8, 4, 4, 2, 2, 1];
/*x delta values*/
pub const ADAM7_DY: [u32; 7] = [8, 8, 8, 4, 4, 2, 2];

pub(super) fn adam7_get_pass_values(
    w: usize,
    h: usize,
    bpp: usize,
) -> ([u32; 7], [u32; 7], [usize; 8], [usize; 8], [usize; 8]) {
    let mut passw: [u32; 7] = [0; 7];
    let mut passh: [u32; 7] = [0; 7];
    let mut filter_passstart: [usize; 8] = [0; 8];
    let mut padded_passstart: [usize; 8] = [0; 8];
    let mut passstart: [usize; 8] = [0; 8];

    /*the passstart values have 8 values: the 8th one indicates the byte after the end of the 7th (= last) pass*/
    /*calculate width and height in pixels of each pass*/
    for i in 0..7 {
        passw[i] = (w as u32 + ADAM7_DX[i] - ADAM7_IX[i] - 1) / ADAM7_DX[i]; /*if passw[i] is 0, it's 0 bytes, not 1 (no filter_type-byte)*/
        passh[i] = (h as u32 + ADAM7_DY[i] - ADAM7_IY[i] - 1) / ADAM7_DY[i]; /*bits padded if needed to fill full byte at end of each scanline*/
        if passw[i] == 0 {
            passh[i] = 0; /*only padded at end of reduced image*/
        }
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
                passh[i] as usize * (1 + (passw[i] as usize * bpp + 7) / 8)
            } else {
                0
            };
        padded_passstart[i + 1] = padded_passstart[i]
            + passh[i] as usize * ((passw[i] as usize * bpp + 7) / 8) as usize;
        passstart[i + 1] = passstart[i]
            + (passh[i] as usize * passw[i] as usize * bpp + 7) / 8;
    }
    (passw, passh, filter_passstart, padded_passstart, passstart)
}

/*
in: Adam7 interlaced image, with no padding bits between scanlines, but between
 reduced images so that each reduced image starts at a byte.
out: the same pixels, but re-ordered so that they're now a non-interlaced image with size w*h
bpp: bits per pixel
out has the following size in bits: w * h * bpp.
in is possibly bigger due to padding bits between reduced images.
out must be big enough AND must be 0 everywhere if bpp < 8 in the current implementation
(because that's likely a little bit faster)
NOTE: comments about padding bits are only relevant if bpp < 8
*/
pub(super) fn adam7_deinterlace(
    out: &mut [u8],
    inp: &[u8],
    w: usize,
    h: usize,
    bpp: usize,
) {
    let (passw, passh, _, _, passstart) = adam7_get_pass_values(w, h, bpp);
    if bpp >= 8 {
        for i in 0..7 {
            let bytewidth = bpp / 8;
            for y in 0..passh[i] {
                for x in 0..passw[i] {
                    let pixelinstart =
                        passstart[i] + (y * passw[i] + x) as usize * bytewidth;
                    let pixeloutstart =
                        ((ADAM7_IY[i] + y * ADAM7_DY[i]) as usize * w
                            + ADAM7_IX[i] as usize
                            + x as usize * ADAM7_DX[i] as usize)
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
            let ilinebits = bpp * passw[i] as usize;
            let olinebits = bpp * w;
            for y in 0..passh[i] as usize {
                for x in 0..passw[i] as usize {
                    let mut ibp =
                        (8 * passstart[i]) + (y * ilinebits + x * bpp) as usize;
                    let mut obp = ((ADAM7_IY[i] as usize
                        + y * ADAM7_DY[i] as usize)
                        * olinebits
                        + (ADAM7_IX[i] as usize + x * ADAM7_DX[i] as usize)
                            * bpp) as usize;
                    for _ in 0..bpp {
                        let bit = read_bit_from_reversed_stream(&mut ibp, inp);
                        /*note that this function assumes the out buffer is completely 0, use set_bit_of_reversed_stream otherwise*/
                        set_bit_of_reversed_stream0(&mut obp, out, bit);
                    }
                }
            }
        }
    };
}
