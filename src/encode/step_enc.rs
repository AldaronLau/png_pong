use std::{any::TypeId, io::Write};

use pix::{
    el::Pixel,
    gray::{SGray16, SGray8, SGraya16, SGraya8},
    rgb::{SRgb16, SRgb8, SRgba16, SRgba8},
    Raster,
};

use crate::{
    adam7,
    bitstream::{BitstreamReader, BitstreamWriter},
    chunk::{
        ColorType, ImageData, ImageEnd, ImageHeader, Palette as PaletteChunk,
        Transparency,
    },
    encode::{filter, ChunkEnc, Error as EncoderError, FilterStrategy, Result},
    encoder::Enc,
    PngRaster, Step,
};

pub trait AsRaster {
    fn get_header(&self, interlace: bool) -> ImageHeader;
    fn get_u8_slice(&self) -> &[u8];
    fn get_palette_colors(&self) -> &[SRgb8];
    fn get_palette_alphas(&self) -> &[u8];
}

impl AsRaster for PngRaster {
    fn get_header(&self, interlace: bool) -> ImageHeader {
        self.header(interlace)
    }

    fn get_u8_slice(&self) -> &[u8] {
        use PngRaster::*;
        match self {
            Rgb8(r) => r.as_u8_slice(),
            Rgba8(r) => r.as_u8_slice(),
            Rgb16(r) => r.as_u8_slice(),
            Rgba16(r) => r.as_u8_slice(),
            Gray8(r) => r.as_u8_slice(),
            Gray16(r) => r.as_u8_slice(),
            Graya8(r) => r.as_u8_slice(),
            Graya16(r) => r.as_u8_slice(),
            Palette(r, _palc, _pala) => r.as_u8_slice(),
        }
    }

    fn get_palette_colors(&self) -> &[SRgb8] {
        use PngRaster::*;
        match self {
            Palette(_r, palc, _pala) => palc.colors(),
            _ => &[],
        }
    }

    fn get_palette_alphas(&self) -> &[u8] {
        use PngRaster::*;
        match self {
            Palette(_r, _palc, pala) => pala.as_slice(),
            _ => &[],
        }
    }
}

impl<P: Pixel> AsRaster for Raster<P> {
    fn get_header(&self, interlace: bool) -> ImageHeader {
        let (color_type, bit_depth) =
            if TypeId::of::<SGray8>() == TypeId::of::<P>() {
                (ColorType::Grey, 8)
            } else if TypeId::of::<SGray16>() == TypeId::of::<P>() {
                (ColorType::Grey, 16)
            } else if TypeId::of::<SGraya8>() == TypeId::of::<P>() {
                (ColorType::GreyAlpha, 8)
            } else if TypeId::of::<SGraya16>() == TypeId::of::<P>() {
                (ColorType::GreyAlpha, 16)
            } else if TypeId::of::<SRgb8>() == TypeId::of::<P>() {
                (ColorType::Rgb, 8)
            } else if TypeId::of::<SRgb16>() == TypeId::of::<P>() {
                (ColorType::Rgb, 16)
            } else if TypeId::of::<SRgba8>() == TypeId::of::<P>() {
                (ColorType::Rgba, 8)
            } else if TypeId::of::<SRgba16>() == TypeId::of::<P>() {
                (ColorType::Rgba, 16)
            } else {
                panic!("Invalid Color Type + Bit Depth Combination For PNG");
            };
        ImageHeader {
            width: self.width(),
            height: self.height(),
            color_type,
            bit_depth,
            interlace,
        }
    }

    fn get_u8_slice(&self) -> &[u8] {
        self.as_u8_slice()
    }

    fn get_palette_colors(&self) -> &[SRgb8] {
        &[]
    }

    fn get_palette_alphas(&self) -> &[u8] {
        &[]
    }
}

/// Frame Encoder for PNG files.
#[derive(Debug)]
pub struct StepEnc<W: Write> {
    encoder: ChunkEnc<W>,
    // FIXME
    #[allow(dead_code)]
    coldepth: Option<(ColorType, u32)>,
    #[allow(dead_code)]
    header: Option<ImageHeader>,
}

impl<W: Write> StepEnc<W> {
    /// Create a new encoder.
    pub(crate) fn new(encoder: ChunkEnc<W>) -> Self {
        Self {
            encoder,
            coldepth: None,
            header: None,
        }
    }

    /// Encode a still (takes either a `png_pong::PngRaster` or `pix::Raster`).
    pub fn still<R: AsRaster>(&mut self, raster: &R) -> Result<()> {
        let image_header = raster.get_header(self.encoder.enc.interlace());

        encode(
            &mut self.encoder.enc,
            raster.get_u8_slice(),
            &image_header,
            raster.get_palette_colors(),
            raster.get_palette_alphas(),
        )
    }

    /// Encode one [`Step`](struct.Step.html) of an animation.
    pub fn encode(&mut self, frame: &Step) -> Result<()> {
        self.still(&frame.raster)
    }
}

pub(super) fn encode<W: Write>(
    enc: &mut Enc<W>,
    image: &[u8],
    header: &ImageHeader,
    palette: &[SRgb8],
    transparency: &[u8],
) -> Result<()> {
    enc.raw(&crate::consts::PNG_SIGNATURE)?;

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

    let data = pre_process_scanlines(
        image,
        header,
        enc.filter_strategy(),
        enc.level(),
    );

    header.write(enc)?;

    if header.color_type == ColorType::Palette {
        let palette = PaletteChunk {
            palette: palette.to_vec(),
        };

        palette.write(enc)?;
    }
    if header.color_type == ColorType::Palette && transparency.len() != 0 {
        transparency.write(enc)?;
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
    ImageData::with_data(data).write(enc)?;
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
    ImageEnd.write(enc)
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
    let diff = olinebits - ilinebits; /* bit pointers */
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
) -> Vec<u8> {
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
        /* image size plus an extra byte per scanline + possible padding bits */
        if bpp < 8 && w * bpp != ((w * bpp + 7) / 8) * 8 {
            let mut padded = vec![0u8; h * ((w * bpp + 7) / 8)]; /* we can immediately filter into the out buffer, no other steps
                                                                  * needed */
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
            );
        } else {
            filter::filter(&mut out, inp, w, h, header, filter_strategy, level);
        }
        out
    } else {
        let (passw, passh, filter_passstart, padded_passstart, passstart) =
            adam7::get_pass_values(width, height, bpp);
        let outsize = filter_passstart[7];
        /* image size plus an extra byte per scanline + possible padding bits */
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
                );
            } else {
                filter::filter(
                    &mut out[filter_passstart[i] as usize..],
                    &adam7[padded_passstart[i] as usize..],
                    passw[i] as usize,
                    passh[i] as usize,
                    header,
                    filter_strategy,
                    level,
                );
            }
        }
        out
    }
}
