// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::{
    adam7,
    bitstream::{BitstreamReader, BitstreamWriter},
    chunk::{ColorType, ImageHeader},
    chunk::{ImageData, ImageEnd, Palette as PaletteChunk, Transparency},
    encode::ChunkEncoder,
    encode::Error as EncoderError,
    filter, FilterStrategy, PngRaster, Step,
};
use pix::rgb::SRgb8;
use std::io::{self, Write};

/// Frame Encoder for PNG files.
#[derive(Debug)]
pub struct StepEncoder<W: Write> {
    encoder: ChunkEncoder<W>,
    coldepth: Option<(ColorType, u32)>,
    filter_strategy: Option<FilterStrategy>,
    interlace: bool,
    header: Option<ImageHeader>,
}

impl<W: Write> StepEncoder<W> {
    /// Create a new encoder.
    pub fn new(
        w: W,
        filter_strategy: Option<FilterStrategy>,
        level: u8,
    ) -> Self {
        Self {
            encoder: ChunkEncoder::new(w, level),
            coldepth: None,
            filter_strategy,
            interlace: false, // FIXME: add parameter via builder
            header: None,
        }
    }

    /// Encode a still.
    pub fn still(&mut self, raster: &PngRaster) -> io::Result<()> {
        use PngRaster::*;
        let fs = self.filter_strategy;
        let image_header = raster.header(self.interlace);

        let bytes = match raster {
            Rgb8(r) => encode(
                r.as_u8_slice(),
                &image_header,
                fs,
                &[],
                &[],
                self.encoder.level,
            ),
            Rgba8(r) => encode(
                r.as_u8_slice(),
                &image_header,
                fs,
                &[],
                &[],
                self.encoder.level,
            ),
            Rgb16(r) => encode(
                r.as_u8_slice(),
                &image_header,
                fs,
                &[],
                &[],
                self.encoder.level,
            ),
            Rgba16(r) => encode(
                r.as_u8_slice(),
                &image_header,
                fs,
                &[],
                &[],
                self.encoder.level,
            ),
            Gray8(r) => encode(
                r.as_u8_slice(),
                &image_header,
                fs,
                &[],
                &[],
                self.encoder.level,
            ),
            Gray16(r) => encode(
                r.as_u8_slice(),
                &image_header,
                fs,
                &[],
                &[],
                self.encoder.level,
            ),
            Graya8(r) => encode(
                r.as_u8_slice(),
                &image_header,
                fs,
                &[],
                &[],
                self.encoder.level,
            ),
            Graya16(r) => encode(
                r.as_u8_slice(),
                &image_header,
                fs,
                &[],
                &[],
                self.encoder.level,
            ),
            Palette(r, pal_rgb, pal_a) => encode(
                r.as_u8_slice(),
                &image_header,
                fs,
                pal_rgb.colors(),
                pal_a.as_slice(),
                self.encoder.level,
            ),
        };
        let bytes = match bytes {
            Ok(o) => o,
            Err(e) => panic!("Encoding failure bug: {:?}!", e),
        };
        match self.encoder.bytes.write(&bytes) {
            Ok(_size) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Encode one [`Step`](struct.Step.html) of an animation.
    pub fn encode(&mut self, frame: &Step) -> io::Result<()> {
        self.still(&frame.raster)
    }
}

pub(super) fn encode(
    image: &[u8],
    header: &ImageHeader,
    filter_strategy: Option<FilterStrategy>,
    palette: &[SRgb8],
    transparency: &[u8],
    level: u8,
) -> Result<Vec<u8>, EncoderError> {
    let transparency = Transparency::Palette(transparency.to_vec());

    if header.color_type == ColorType::Palette
        && (palette.is_empty() || palette.len() > 256)
    {
        return Err(EncoderError::BadPalette);
    }
    header
        .color_type
        .check_png_color_validity(header.bit_depth)
        .unwrap();

    let data = pre_process_scanlines(image, header, filter_strategy, level)?;

    let mut outv = crate::consts::PNG_SIGNATURE.to_vec();

    header.write(&mut outv)?;

    if header.color_type == ColorType::Palette {
        let palette = PaletteChunk {
            palette: palette.to_vec(),
        };

        palette.write(&mut outv)?;
    }
    if header.color_type == ColorType::Palette && transparency.len() != 0 {
        transparency.write(&mut outv)?;
    }
    // FIXME: Transparency KEY
    /*if color_type == ColorType::Grey
        && discriminant(transparency)
            == discriminant(&Transparency::GrayKey(SRgb16::default()))
    {
        transparency.write(&mut outv)?;
    }
    if color_type == ColorType::Rgb
        && discriminant(transparency)
            == discriminant(&Transparency::RgbKey(0, 0, 0))
    {
        transparency.write(&mut outv)?;
    }*/
    // FIXME
    /*if let Some(ref background) = background {
        background.write(&mut outv, color_type)?;
    }
    if let Some(ref physical) = info.phys {
        Physical::write(&mut outv, physical)?;
    }*/
    /*if let Some(_chunks) = info.unknown_chunks_data(ChunkPosition::PLTE) {
        // add_unknown_chunks(&mut outv, _chunks);
    }*/
    ImageData::with_data(data).write(&mut outv, level)?;
    /*if let Some(ref time) = info.time {
        time.write(&mut outv)?;
    }*/
    // FIXME: Text
    /*for ntext in info.text.iter() {
        if ntext.key.len() > 79 || ntext.key.is_empty() {
            return Err(EncoderError::TextSize(ntext.key.len()));
        }
        Text {
            key: ntext.key.clone(),
            val: ntext.val.clone(),
        }
        .write(&mut outv)?;
    }
    for ztext in info.ztext.iter() {
        if ztext.key.len() > 79 || ztext.key.is_empty() {
            return Err(EncoderError::TextSize(ztext.key.len()));
        }
        ZText {
            key: ztext.key.clone(),
            val: ztext.val.clone(),
        }
        .write(&mut outv, zlib_compression)?;
    }
    for chunk in info.itext.iter() {
        if chunk.key.len() > 79 || chunk.key.is_empty() {
            return Err(EncoderError::TextSize(chunk.key.len()));
        }
        chunk.write(
            &mut outv,
            text_compression,
            zlib_compression,
        )?;
    }*/
    /*if let Some(_chunks) = info.unknown_chunks_data(ChunkPosition::IDAT) {
        // add_unknown_chunks(&mut outv, _chunks);
    }*/
    ImageEnd.write(&mut outv)?;
    Ok(outv)
}

/// The opposite of the remove_padding_bits function
/// olinebits must be >= ilinebits
fn add_padding_bits(
    out: &mut [u8],
    inp: &[u8],
    olinebits: usize,
    ilinebits: usize,
    h: usize,
) {
    let diff = olinebits - ilinebits; /*bit pointers*/
    let mut out_buf = Vec::with_capacity(h * ((ilinebits + diff) / 8));
    let mut out_stream = BitstreamWriter::new(&mut out_buf);
    let mut in_stream =
        BitstreamReader::new(std::io::Cursor::new(inp)).unwrap();
    for _ in 0..h {
        for _ in 0..ilinebits {
            let bit = in_stream.read().unwrap().unwrap();
            out_stream.write(bit).unwrap();
        }
        for _ in 0..diff {
            out_stream.write(false).unwrap();
        }
    }
    // Copy output buffer into array.
    for (i, byte) in out_buf.iter().cloned().enumerate() {
        out[i] = byte;
    }
}

// Out must be buffer big enough to contain uncompressed IDAT chunk data, and in
// must contain the full image.
fn pre_process_scanlines(
    inp: &[u8],
    header: &ImageHeader,
    filter_strategy: Option<FilterStrategy>,
    level: u8,
) -> Result<Vec<u8>, EncoderError> {
    let width = header.width;
    let height = header.height;
    let bit_depth = header.bit_depth;
    let color_type = header.color_type;
    let h = height as usize;
    let w = width as usize;
    let bpp = color_type.bpp(bit_depth);
    /*
    This function converts the pure 2D image with the PNG's colortype, into filtered-padded-interlaced data. Steps:
    *) if no Adam7: 1) add padding bits (= posible extra bits per scanline if bpp < 8) 2) filter
    *) if adam7: 1) adam7_interlace 2) 7x add padding bits 3) 7x filter
    */

    if !header.interlace {
        let bpp = bpp as usize;
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
            filter::filter(
                &mut out,
                &padded,
                w,
                h,
                header,
                filter_strategy,
                level,
            )?;
        } else {
            filter::filter(
                &mut out,
                inp,
                w,
                h,
                header,
                filter_strategy,
                level,
            )?;
        }
        Ok(out)
    } else {
        let (passw, passh, filter_passstart, padded_passstart, passstart) =
            adam7::get_pass_values(width, height, bpp);
        let outsize = filter_passstart[7];
        /*image size plus an extra byte per scanline + possible padding bits*/
        let mut out = vec![0u8; outsize as usize];
        let mut adam7 = vec![0u8; passstart[7] as usize + 1];
        adam7::interlace(&mut adam7, inp, width, height, bpp);
        let bpp = bpp as usize;
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
                filter::filter(
                    &mut out[filter_passstart[i] as usize..],
                    &padded,
                    passw[i] as usize,
                    passh[i] as usize,
                    header,
                    filter_strategy,
                    level,
                )?;
            } else {
                filter::filter(
                    &mut out[filter_passstart[i] as usize..],
                    &adam7[padded_passstart[i] as usize..],
                    passw[i] as usize,
                    passh[i] as usize,
                    header,
                    filter_strategy,
                    level,
                )?;
            }
        }
        Ok(out)
    }
}
