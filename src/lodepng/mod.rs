#![allow(unused)] // TODO: Just for now.

use miniz_oxide::deflate::compress_to_vec;

use pix::Alpha;

mod ffi;

mod rustimpl;
use rustimpl::*;

mod error;
pub use error::*;
mod iter;
use iter::*;

use std::cmp;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::ptr;

pub use ffi::ColorType;
pub use ffi::CompressSettings;
pub(crate) use ffi::DecoderSettings;
pub(crate) use ffi::DecompressSettings;
pub use ffi::EncoderSettings;
pub use ffi::Error;
pub use ffi::FilterStrategy;
pub(crate) use ffi::State;
pub use ffi::Time;

pub use ffi::ColorMode;
pub use ffi::Info;

use crate::prelude::*;

impl ColorMode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn colortype(&self) -> ColorType {
        self.colortype
    }

    #[inline]
    pub fn bitdepth(&self) -> u32 {
        self.bitdepth
    }

    pub fn set_bitdepth(&mut self, d: u32) {
        assert!(d >= 1 && d <= 16);
        self.bitdepth = d;
    }

    pub fn palette_clear(&mut self) {
        self.palette = Vec::with_capacity(256);
    }

    /// add 1 color to the palette
    pub fn palette_add(&mut self, p: Rgba8) -> Result<(), Error> {
        if self.palette.len() >= 256 {
            return Err(Error(38));
        }
        self.palette.push(p);

        Ok(())
    }

    pub fn palette(&self) -> &[Rgba8] {
        self.palette.as_slice()
    }

    pub fn palette_mut(&mut self) -> &mut [Rgba8] {
        self.palette.as_mut_slice()
    }

    /// get the total amount of bits per pixel, based on colortype and bitdepth in the struct
    pub fn bpp(&self) -> u32 {
        lodepng_get_bpp_lct(self.colortype, self.bitdepth()) /*4 or 6*/
    }

    pub(crate) fn clear_key(&mut self) {
        self.key = None;
    }

    pub(crate) fn set_key(&mut self, r: u16, g: u16, b: u16) {
        self.key = Some((u32::from(r), u32::from(g), u32::from(b)));
    }

    pub(crate) fn key(&self) -> Option<(u16, u16, u16)> {
        if let Some((r, g, b)) = self.key {
            Some((r as u16, g as u16, b as u16))
        } else {
            None
        }
    }

    /// get the amount of color channels used, based on colortype in the struct.
    /// If a palette is used, it counts as 1 channel.
    pub fn channels(&self) -> u8 {
        self.colortype.channels()
    }

    /// is it a greyscale type? (only colortype 0 or 4)
    pub fn is_greyscale_type(&self) -> bool {
        self.colortype == ColorType::Grey
            || self.colortype == ColorType::GreyAlpha
    }

    /// has it got an alpha channel? (only colortype 2 or 6)
    pub fn is_alpha_type(&self) -> bool {
        (self.colortype as u32 & 4) != 0
    }

    /// has it got a palette? (only colortype 3)
    pub fn is_palette_type(&self) -> bool {
        self.colortype == ColorType::Palette
    }

    /// only returns true if there is a palette and there is a value in the palette with alpha < 255.
    /// Loops through the palette to check this.
    pub fn has_palette_alpha(&self) -> bool {
        self.palette().iter().any(|p| {
            let alpha = p.alpha();
            let value = alpha.value();
            let byte: u8 = value.into();

            byte < 255
        })
    }

    /// Check if the given color info indicates the possibility of having non-opaque pixels in the PNG image.
    /// Returns true if the image can have translucent or invisible pixels (it still be opaque if it doesn't use such pixels).
    /// Returns false if the image can only have opaque pixels.
    /// In detail, it returns true only if it's a color type with alpha, or has a palette with non-opaque values,
    /// or if "key_defined" is true.
    pub fn can_have_alpha(&self) -> bool {
        self.key().is_some() || self.is_alpha_type() || self.has_palette_alpha()
    }

    /// Returns the byte size of a raw image buffer with given width, height and color mode
    pub fn raw_size(&self, w: u32, h: u32) -> usize {
        /*will not overflow for any color type if roughly w * h < 268435455*/
        let bpp = self.bpp() as usize;
        let n = w as usize * h as usize;
        ((n / 8) * bpp) + ((n & 7) * bpp + 7) / 8
    }

    /*in an idat chunk, each scanline is a multiple of 8 bits, unlike the lodepng output buffer*/
    pub(crate) fn raw_size_idat(&self, w: usize, h: usize) -> usize {
        /*will not overflow for any color type if roughly w * h < 268435455*/
        let bpp = self.bpp() as usize;
        let line = ((w / 8) * bpp) + ((w & 7) * bpp + 7) / 8;
        h * line
    }
}

impl Drop for ColorMode {
    fn drop(&mut self) {
        self.palette_clear()
    }
}

impl Clone for ColorMode {
    fn clone(&self) -> Self {
        let mut c = Self {
            colortype: self.colortype,
            bitdepth: self.bitdepth,
            palette: Vec::with_capacity(256),
            key: self.key,
        };
        for &p in self.palette() {
            c.palette_add(p).unwrap();
        }
        c
    }
}

impl Default for ColorMode {
    fn default() -> Self {
        Self {
            key: None,
            colortype: ColorType::Rgba,
            bitdepth: 8,
            palette: Vec::with_capacity(256),
        }
    }
}

impl ColorType {
    /// Create color mode with given type and bitdepth
    pub fn to_color_mode(&self, bitdepth: u32) -> ColorMode {
        ColorMode {
            colortype: *self,
            bitdepth,
            palette: Vec::new(),
            key: None,
        }
    }

    /// channels * bytes per channel = bytes per pixel
    pub fn channels(&self) -> u8 {
        match *self {
            ColorType::Grey | ColorType::Palette => 1,
            ColorType::GreyAlpha => 2,
            ColorType::Bgr | ColorType::Rgb => 3,
            ColorType::Bgra | ColorType::Bgrx | ColorType::Rgba => 4,
        }
    }
}

impl Time {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Info {
    pub fn new() -> Self {
        Self {
            color: ColorMode::new(),
            interlace_method: 0,
            compression_method: 0,
            filter_method: 0,
            background_defined: 0,
            background_r: 0,
            background_g: 0,
            background_b: 0,
            time_defined: 0,
            time: Time::new(),
            unknown_chunks_data: [Vec::new(), Vec::new(), Vec::new()],
            text: Vec::new(),
            itext: Vec::new(),
            phys_defined: 0,
            phys_x: 0,
            phys_y: 0,
            phys_unit: 0,
        }
    }

    pub fn text_keys_cstr(&self) -> std::slice::Iter<'_, TextChunk> {
        self.text.iter()
    }

    pub fn itext_keys(&self) -> std::slice::Iter<'_, ITextChunk> {
        self.itext.iter()
    }

    /// use this to clear the texts again after you filled them in
    pub fn clear_text(&mut self) {
        self.text.clear();
    }

    /// push back both texts at once
    pub fn add_text(&mut self, key: &str, str: &str) {
        self.push_text(key.as_bytes(), str.as_bytes());
    }

    /// use this to clear the itexts again after you filled them in
    pub fn clear_itext(&mut self) {
        self.itext.clear();
    }

    /// push back the 4 texts of 1 chunk at once
    pub fn add_itext(
        &mut self,
        key: &str,
        langtag: &str,
        transkey: &str,
        text: &str,
    ) -> Result<(), Error> {
        self.push_itext(
            key.as_bytes(),
            langtag.as_bytes(),
            transkey.as_bytes(),
            text.as_bytes(),
        )
    }

    pub fn append_chunk(
        &mut self,
        position: ChunkPosition,
        chunk: ChunkRef,
    ) -> Result<(), Error> {
        let set = position as usize;
        let mut tmp = self.unknown_chunks_data[set].clone();

        chunk_append(&mut tmp, chunk.data);
        self.unknown_chunks_data[set] = tmp;

        Ok(())
    }

    pub fn create_chunk<C: AsRef<[u8]>>(
        &mut self,
        position: ChunkPosition,
        chtype: C,
        data: &[u8],
    ) -> Result<(), Error> {
        let chtype = chtype.as_ref();
        if chtype.len() != 4 {
            return Err(Error(67));
        }

        let type_: [u8; 4] = [chtype[0], chtype[1], chtype[2], chtype[3]];

        rustimpl::add_chunk(
            &mut self.unknown_chunks_data[position as usize],
            &type_,
            data,
        )
    }

    pub fn get<Name: AsRef<[u8]>>(&self, index: Name) -> Option<ChunkRef> {
        let index = index.as_ref();
        self.unknown_chunks(ChunkPosition::IHDR)
            .chain(self.unknown_chunks(ChunkPosition::PLTE))
            .chain(self.unknown_chunks(ChunkPosition::IDAT))
            .find(|c| c.is_type(index))
    }

    pub fn unknown_chunks(&self, position: ChunkPosition) -> ChunksIter {
        ChunksIter {
            data: self.unknown_chunks_data[position as usize].as_slice(),
        }
    }

    fn set_unknown_chunks(&mut self, src: &Info) -> Result<(), Error> {
        for i in 0..3 {
            self.unknown_chunks_data[i] =
                Vec::with_capacity(src.unknown_chunks_data[i].len());

            for j in 0..src.unknown_chunks_data[i].len() {
                self.unknown_chunks_data[i].push(src.unknown_chunks_data[i][j])
            }
        }
        Ok(())
    }
}

impl Clone for Info {
    fn clone(&self) -> Self {
        let mut dest = Self {
            compression_method: self.compression_method,
            filter_method: self.filter_method,
            interlace_method: self.interlace_method,
            color: self.color.clone(),
            background_defined: self.background_defined,
            background_r: self.background_r,
            background_g: self.background_g,
            background_b: self.background_b,
            text: Vec::new(),
            itext: Vec::new(),
            time_defined: self.time_defined,
            time: self.time,
            phys_defined: self.phys_defined,
            phys_x: self.phys_x,
            phys_y: self.phys_y,
            phys_unit: self.phys_unit,
            unknown_chunks_data: [Vec::new(), Vec::new(), Vec::new()],
        };
        rustimpl::text_copy(&mut dest, self).unwrap();
        rustimpl::itext_copy(&mut dest, self).unwrap();
        dest.set_unknown_chunks(self).unwrap();
        dest
    }
}

#[derive(Clone, Debug, Default)]
/// Make an image with custom settings
pub struct Encoder {
    state: State,
}

impl Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn set_auto_convert(&mut self, mode: bool) {
        self.state.set_auto_convert(mode);
    }

    #[inline]
    pub fn set_filter_strategy(
        &mut self,
        mode: FilterStrategy,
        palette_filter_zero: bool,
    ) {
        self.state.set_filter_strategy(mode, palette_filter_zero);
    }

    #[inline]
    pub fn info_raw(&self) -> &ColorMode {
        self.state.info_raw()
    }

    #[inline]
    /// Color mode of the source bytes to be encoded
    pub fn info_raw_mut(&mut self) -> &mut ColorMode {
        self.state.info_raw_mut()
    }

    #[inline]
    pub fn info_png(&self) -> &Info {
        self.state.info_png()
    }

    #[inline]
    /// Color mode of the file to be created
    pub fn info_png_mut(&mut self) -> &mut Info {
        self.state.info_png_mut()
    }

    #[inline]
    pub fn encode<PixelType: Copy + pix::Format>(
        &mut self,
        raster: &pix::Raster<PixelType>,
    ) -> Result<Vec<u8>, Error> {
        self.state.encode(raster)
    }

    #[inline]
    pub fn encode_file<PixelType: Copy + pix::Format, P: AsRef<Path>>(
        &mut self,
        filepath: P,
        raster: &pix::Raster<PixelType>,
    ) -> Result<(), Error> {
        self.state.encode_file(filepath, raster)
    }
}

#[derive(Clone, Debug, Default)]
/// Read an image with custom settings
pub struct Decoder {
    state: State,
}

impl Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn info_raw(&self) -> &ColorMode {
        self.state.info_raw()
    }

    #[inline]
    /// Preferred color mode for decoding
    pub fn info_raw_mut(&mut self) -> &mut ColorMode {
        self.state.info_raw_mut()
    }

    #[inline]
    /// Actual color mode of the decoded image or inspected file
    pub fn info_png(&self) -> &Info {
        self.state.info_png()
    }

    #[inline]
    pub fn info_png_mut(&mut self) -> &mut Info {
        self.state.info_png_mut()
    }

    /// whether to convert the PNG to the color type you want. Default: yes
    pub fn color_convert(&mut self, true_or_false: bool) {
        self.state.color_convert(true_or_false);
    }

    /// Decompress ICC profile from iCCP chunk
    pub fn get_icc(&self) -> Result<Vec<u8>, Error> {
        self.state.get_icc()
    }

    /// Load PNG from buffer using State's settings
    ///
    //  ```no_run
    //  # use png_pong::*; let mut state = State::new();
    //  # let slice = [0u8]; #[allow(unused_variables)] fn do_stuff<T>(_buf: T) {}
    //
    //  state.info_raw_mut().colortype = ColorType::Rgba;
    //  match state.decode(&slice) {
    //      Ok(Image::RGBA(with_alpha)) => do_stuff(with_alpha),
    //      _ => panic!("¯\\_(ツ)_/¯")
    //  }
    //  ```
    #[inline]
    pub(crate) fn decode<Bytes: AsRef<[u8]>>(
        &mut self,
        input: Bytes,
    ) -> Result<Image, Error> {
        self.state.decode(input)
    }

    pub fn decode_file<P: AsRef<Path>>(
        &mut self,
        filepath: P,
    ) -> Result<Image, Error> {
        self.state.decode_file(filepath)
    }

    /// Updates `info_png`. Returns (width, height)
    pub fn inspect(&mut self, input: &[u8]) -> Result<(usize, usize), Error> {
        self.state.inspect(input)
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_auto_convert(&mut self, mode: bool) {
        self.encoder.auto_convert = mode as u32;
    }

    pub fn set_filter_strategy(
        &mut self,
        mode: FilterStrategy,
        palette_filter_zero: bool,
    ) {
        self.encoder.filter_strategy = mode;
        self.encoder.filter_palette_zero =
            if palette_filter_zero { 1 } else { 0 };
    }

    pub fn info_raw(&self) -> &ColorMode {
        &self.info_raw
    }

    pub fn info_raw_mut(&mut self) -> &mut ColorMode {
        &mut self.info_raw
    }

    pub fn info_png(&self) -> &Info {
        &self.info_png
    }

    pub fn info_png_mut(&mut self) -> &mut Info {
        &mut self.info_png
    }

    /// whether to convert the PNG to the color type you want. Default: yes
    pub fn color_convert(&mut self, true_or_false: bool) {
        self.decoder.color_convert = if true_or_false { 1 } else { 0 };
    }

    /// Decompress ICC profile from iCCP chunk
    pub fn get_icc(&self) -> Result<Vec<u8>, Error> {
        let iccp = self.info_png().get("iCCP");
        if iccp.is_none() {
            return Err(Error(89));
        }
        let iccp = iccp.as_ref().unwrap().data();
        if iccp.get(0).cloned().unwrap_or(255) == 0 {
            // text min length is 1
            return Err(Error(89));
        }

        let name_len = cmp::min(iccp.len(), 80); // skip name
        for i in 0..name_len {
            if iccp[i] == 0 {
                // string terminator
                if iccp.get(i + 1).cloned().unwrap_or(255) != 0 {
                    // compression type
                    return Err(Error(72));
                }
                return zlib_decompress(
                    &iccp[i + 2..],
                    &self.decoder.zlibsettings,
                );
            }
        }
        Err(Error(75))
    }

    /// Load PNG from buffer using State's settings
    ///
    //  ```no_run
    //  # use png_pong::*; let mut state = State::new();
    //  # let slice = [0u8]; #[allow(unused_variables)] fn do_stuff<T>(_buf: T) {}
    //
    //  state.info_raw_mut().colortype = ColorType::Rgba;
    //  match state.decode(&slice) {
    //      Ok(Image::RGBA(with_alpha)) => do_stuff(with_alpha),
    //      _ => panic!("¯\\_(ツ)_/¯")
    //  }
    //  ```
    pub(crate) fn decode<Bytes: AsRef<[u8]>>(
        &mut self,
        input: Bytes,
    ) -> Result<Image, Error> {
        let input = input.as_ref();
        let (v, w, h) = rustimpl::lodepng_decode(self, input)?;

        Ok(new_bitmap(
            v,
            w,
            h,
            self.info_raw.colortype,
            self.info_raw.bitdepth,
        ))
    }

    pub fn decode_file<P: AsRef<Path>>(
        &mut self,
        filepath: P,
    ) -> Result<Image, Error> {
        self.decode(&load_file(filepath)?)
    }

    /// Updates `info_png`. Returns (width, height)
    pub fn inspect(&mut self, input: &[u8]) -> Result<(usize, usize), Error> {
        let (info, w, h) = rustimpl::lodepng_inspect(&self.decoder, input)?;
        self.info_png = info;
        Ok((w, h))
    }

    pub fn encode<PixelType: Copy + pix::Format>(
        &mut self,
        raster: &pix::Raster<PixelType>,
    ) -> Result<Vec<u8>, Error> {
        Ok(rustimpl::lodepng_encode(
            raster.as_u8_slice(),
            raster.width() as u32,
            raster.height() as u32,
            self,
        )?)
    }

    pub fn encode_file<PixelType: Copy + pix::Format, P: AsRef<Path>>(
        &mut self,
        filepath: P,
        raster: &pix::Raster<PixelType>,
    ) -> Result<(), Error> {
        let buf = self.encode(raster)?;
        save_file(filepath, buf.as_ref())
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            decoder: DecoderSettings::new(),
            encoder: EncoderSettings::new(),
            info_raw: ColorMode::new(),
            info_png: Info::new(),
            error: Error(1),
        }
    }
}

/// Bitmap types.
///
/// Images with >=8bpp are stored with pixel per vec element.
/// Images with <8bpp are represented as a bunch of bytes, with multiple pixels per byte.
pub enum Image {
    /// Bytes of the image. See bpp how many pixels per element there are
    RawData(pix::Raster<pix::Mask8>),
    Grey(pix::Raster<pix::Gray8>),
    Grey16(pix::Raster<pix::Gray16>),
    GreyAlpha(pix::Raster<pix::GrayAlpha8>),
    GreyAlpha16(pix::Raster<pix::GrayAlpha16>),
    RGBA(pix::Raster<Rgba8>),
    RGB(pix::Raster<Rgb8>),
    RGBA16(pix::Raster<pix::Rgba16>),
    RGB16(pix::Raster<pix::Rgb16>),
}

/// Position in the file section after…
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ChunkPosition {
    IHDR = 0,
    PLTE = 1,
    IDAT = 2,
}

/// Reference to a chunk
#[derive(Copy, Clone)]
pub struct ChunkRef<'a> {
    data: &'a [u8],
}

fn new_bitmap(
    out: Vec<u8>,
    w: usize,
    h: usize,
    colortype: ColorType,
    bitdepth: u32,
) -> Image {
    // TODO as parameters instead of casting.
    let width = w as u32;
    let height = h as u32;

    match (colortype, bitdepth) {
        (ColorType::Rgba, 8) => Image::RGBA(
            pix::RasterBuilder::new().with_u8_buffer(width, height, out),
        ),
        (ColorType::Rgb, 8) => Image::RGB(
            pix::RasterBuilder::new().with_u8_buffer(width, height, out),
        ),
        (ColorType::Rgba, 16) => {
            let out: Vec<u16> = out
                .chunks_exact(2)
                .into_iter()
                .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                .collect();

            Image::RGBA16(
                pix::RasterBuilder::new().with_u16_buffer(width, height, out),
            )
        }
        (ColorType::Rgb, 16) => {
            let out: Vec<u16> = out
                .chunks_exact(2)
                .into_iter()
                .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                .collect();

            Image::RGB16(
                pix::RasterBuilder::new().with_u16_buffer(width, height, out),
            )
        }
        (ColorType::Grey, 8) => Image::Grey(
            pix::RasterBuilder::new().with_u8_buffer(width, height, out),
        ),
        (ColorType::Grey, 16) => {
            let out: Vec<u16> = out
                .chunks_exact(2)
                .into_iter()
                .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                .collect();
            Image::Grey16(
                pix::RasterBuilder::new().with_u16_buffer(width, height, out),
            )
        }
        (ColorType::GreyAlpha, 8) => Image::GreyAlpha(
            pix::RasterBuilder::new().with_u8_buffer(width, height, out),
        ),
        (ColorType::GreyAlpha, 16) => {
            let out: Vec<u16> = out
                .chunks_exact(2)
                .into_iter()
                .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                .collect();
            Image::GreyAlpha16(
                pix::RasterBuilder::new().with_u16_buffer(width, height, out),
            )
        }
        (_, 0) => panic!("Invalid depth"),
        (_c, _b) => Image::RawData(
            pix::RasterBuilder::new().with_u8_buffer(width, height, out),
        ),
    }
}

fn save_file<P: AsRef<Path>>(filepath: P, data: &[u8]) -> Result<(), Error> {
    let mut file = File::create(filepath)?;
    file.write_all(data)?;
    Ok(())
}

fn load_file<P: AsRef<Path>>(filepath: P) -> Result<Vec<u8>, Error> {
    let mut file = File::open(filepath)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

/// Converts PNG data in memory to raw pixel data.
///
/// `decode32` and `decode24` are more convenient if you want specific image format.
///
/// See `State::decode()` for advanced decoding.
///
/// * `in`: Memory buffer with the PNG file.
/// * `colortype`: the desired color type for the raw output image. See `ColorType`.
/// * `bitdepth`: the desired bit depth for the raw output image. 1, 2, 4, 8 or 16. Typically 8.
pub fn decode_memory<Bytes: AsRef<[u8]>>(
    input: Bytes,
    colortype: ColorType,
    bitdepth: u32,
) -> Result<Image, Error> {
    let input = input.as_ref();

    assert!(bitdepth > 0 && bitdepth <= 16);
    let (v, w, h) =
        rustimpl::lodepng_decode_memory(input, colortype, bitdepth)?;
    Ok(new_bitmap(v, w, h, colortype, bitdepth))
}

/// Same as `decode_memory`, but always decodes to 32-bit RGBA raw image
pub fn decode32<Bytes: AsRef<[u8]>>(
    input: Bytes,
) -> Result<pix::Raster<pix::Rgba8>, Error> {
    match decode_memory(input, ColorType::Rgba, 8)? {
        Image::RGBA(img) => Ok(img),
        _ => Err(Error(56)), // given output image colortype or bitdepth not supported for color conversion
    }
}

/// Converts raw pixel data into a PNG image in memory. The colortype and bitdepth
/// of the output PNG image cannot be chosen, they are automatically determined
/// by the colortype, bitdepth and content of the input pixel data.
///
/// Note: for 16-bit per channel colors, needs big endian format like PNG does.
///
/// * `image`: The raw pixel data to encode. The size of this buffer should be `w` * `h` * (bytes per pixel), bytes per pixel depends on colortype and bitdepth.
/// * `w`: width of the raw pixel data in pixels.
/// * `h`: height of the raw pixel data in pixels.
/// * `colortype`: the color type of the raw input image. See `ColorType`.
/// * `bitdepth`: the bit depth of the raw input image. 1, 2, 4, 8 or 16. Typically 8.
pub fn encode_memory<PixelType: Copy + pix::Format>(
    raster: &pix::Raster<PixelType>,
    colortype: ColorType,
    bitdepth: u32,
) -> Result<Vec<u8>, Error> {
    Ok(rustimpl::lodepng_encode_memory(
        raster.as_u8_slice(),
        raster.width(),
        raster.height(),
        colortype,
        bitdepth,
    )?)
}

/// Same as `encode_memory`, but always encodes from 32-bit RGBA raw image
pub fn encode32<PixelType: Copy + pix::Format>(
    raster: &pix::Raster<PixelType>,
) -> Result<Vec<u8>, Error> {
    encode_memory(raster, ColorType::Rgba, 8)
}

/// Same as `encode_memory`, but always encodes from 24-bit RGB raw image
pub fn encode24<PixelType: Copy + pix::Format>(
    raster: &pix::Raster<PixelType>,
) -> Result<Vec<u8>, Error> {
    encode_memory(raster, ColorType::Rgb, 8)
}

/// Converts raw pixel data into a PNG file on disk.
/// Same as the other encode functions, but instead takes a file path as output.
///
/// NOTE: This overwrites existing files without warning!
pub fn encode_file<PixelType: Copy + pix::Format, P: AsRef<Path>>(
    filepath: P,
    raster: &pix::Raster<PixelType>,
    colortype: ColorType,
    bitdepth: u32,
) -> Result<(), Error> {
    let encoded = encode_memory(raster, colortype, bitdepth)?;
    save_file(filepath, encoded.as_ref())
}

/// Same as `encode_file`, but always encodes from 32-bit RGBA raw image
pub fn encode32_file<PixelType: Copy + pix::Format, P: AsRef<Path>>(
    filepath: P,
    raster: &pix::Raster<PixelType>,
) -> Result<(), Error> {
    encode_file(filepath, raster, ColorType::Rgba, 8)
}

/// Same as `encode_file`, but always encodes from 24-bit RGB raw image
pub fn encode24_file<PixelType: Copy + pix::Format, P: AsRef<Path>>(
    filepath: P,
    raster: &pix::Raster<PixelType>,
) -> Result<(), Error> {
    encode_file(filepath, raster, ColorType::Rgb, 8)
}

impl<'a> ChunkRef<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn len(&self) -> usize {
        rustimpl::lodepng_chunk_length(self.data)
    }

    pub fn name(&self) -> [u8; 4] {
        let mut tmp = [0; 4];
        tmp.copy_from_slice(rustimpl::lodepng_chunk_type(self.data));
        tmp
    }

    pub fn is_type<C: AsRef<[u8]>>(&self, name: C) -> bool {
        rustimpl::lodepng_chunk_type(self.data) == name.as_ref()
    }

    pub fn is_ancillary(&self) -> bool {
        rustimpl::lodepng_chunk_ancillary(self.data)
    }

    pub fn is_private(&self) -> bool {
        rustimpl::lodepng_chunk_private(self.data)
    }

    pub fn is_safe_to_copy(&self) -> bool {
        rustimpl::lodepng_chunk_safetocopy(self.data)
    }

    pub fn data(&self) -> &[u8] {
        rustimpl::lodepng_chunk_data(self.data).unwrap()
    }

    pub fn check_crc(&self) -> bool {
        rustimpl::lodepng_chunk_check_crc(&*self.data)
    }
}

pub struct ChunkRefMut<'a> {
    data: &'a mut [u8],
}

impl<'a> ChunkRefMut<'a> {
    pub fn data_mut(&mut self) -> &mut [u8] {
        rustimpl::lodepng_chunk_data_mut(self.data).unwrap()
    }

    pub fn generate_crc(&mut self) {
        rustimpl::lodepng_chunk_generate_crc(self.data)
    }
}

/// Compresses data with Zlib.
/// Zlib adds a small header and trailer around the deflate data.
/// The data is output in the format of the zlib specification.
pub fn zlib_compress(
    input: &[u8],
    settings: &CompressSettings,
) -> Result<Vec<u8>, Error> {
    let mut v = Vec::new();
    rustimpl::lodepng_zlib_compress(&mut v, input, settings)?;
    Ok(v)
}

fn zlib_decompress(
    input: &[u8],
    settings: &DecompressSettings,
) -> Result<Vec<u8>, Error> {
    Ok(rustimpl::lodepng_zlib_decompress(input, settings)?)
}

/// Compress a buffer with deflate. See RFC 1951.
pub fn deflate(
    input: &[u8],
    settings: &CompressSettings,
) -> Result<Vec<u8>, Error> {
    if settings.btype > 2 {
        Err(Error(61))
    } else if settings.btype == 0 {
        Ok(compress_to_vec(input, 0))
    } else {
        Ok(compress_to_vec(input, 10))
    }
}

impl CompressSettings {
    /// Default compression settings
    pub fn new() -> CompressSettings {
        Self::default()
    }
}

impl Default for CompressSettings {
    fn default() -> Self {
        Self {
            btype: 2,
            use_lz77: 1,
            windowsize: DEFAULT_WINDOWSIZE as u32,
            minmatch: 3,
            nicematch: 128,
            lazymatching: 1,
            custom_context: ptr::null_mut(),
        }
    }
}

impl DecompressSettings {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for DecompressSettings {
    fn default() -> Self {
        Self {
            check_adler32: false,
            custom_context: ptr::null_mut(),
        }
    }
}

impl DecoderSettings {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for DecoderSettings {
    fn default() -> Self {
        Self {
            color_convert: 1,
            check_crc: false,
            zlibsettings: DecompressSettings::new(),
        }
    }
}

impl EncoderSettings {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for EncoderSettings {
    fn default() -> Self {
        Self {
            zlibsettings: CompressSettings::new(),
            filter_palette_zero: 1,
            filter_strategy: FilterStrategy::Minsum,
            auto_convert: 1,
            force_palette: 0,
            add_id: 0,
            text_compression: 1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pix::Ch8;
    use std::mem;

    #[test]
    fn pixel_sizes() {
        assert_eq!(4, mem::size_of::<pix::Rgba8>());
        assert_eq!(3, mem::size_of::<pix::Rgb8>());
        assert_eq!(2, mem::size_of::<pix::GrayAlpha8>());
        assert_eq!(1, mem::size_of::<pix::Gray8>());
    }

    #[test]
    fn create_and_destroy1() {
        DecoderSettings::new();
        EncoderSettings::new();
        CompressSettings::new();
    }

    #[test]
    fn create_and_destroy2() {
        State::new().info_png();
        State::new().info_png_mut();
        State::new().clone().info_raw();
        State::new().clone().info_raw_mut();
    }

    #[test]
    fn test_pal() {
        let mut state = State::new();
        state.info_raw_mut().colortype = ColorType::Palette;
        assert_eq!(state.info_raw().colortype(), ColorType::Palette);
        state
            .info_raw_mut()
            .palette_add(pix::Rgba8::with_alpha(
                Ch8::new(1),
                Ch8::new(2),
                Ch8::new(3),
                Ch8::new(4),
            ))
            .unwrap();
        state
            .info_raw_mut()
            .palette_add(pix::Rgba8::with_alpha(
                Ch8::new(5),
                Ch8::new(6),
                Ch8::new(7),
                Ch8::new(255),
            ))
            .unwrap();
        assert_eq!(
            &[
                pix::Rgba8::with_alpha(
                    Ch8::new(1u8),
                    Ch8::new(2),
                    Ch8::new(3),
                    Ch8::new(4)
                ),
                pix::Rgba8::with_alpha(
                    Ch8::new(5u8),
                    Ch8::new(6),
                    Ch8::new(7),
                    Ch8::new(255)
                )
            ],
            state.info_raw().palette()
        );
        state.info_raw_mut().palette_clear();
        assert_eq!(0, state.info_raw().palette().len());
    }

    #[test]
    fn chunks() {
        let mut state = State::new();
        {
            let info = state.info_png_mut();
            for _ in info.unknown_chunks(ChunkPosition::IHDR) {
                panic!("no chunks yet");
            }

            let testdata = &[1, 2, 3];
            info.create_chunk(
                ChunkPosition::PLTE,
                &[255, 0, 100, 32],
                testdata,
            )
            .unwrap();
            assert_eq!(1, info.unknown_chunks(ChunkPosition::PLTE).count());

            info.create_chunk(ChunkPosition::IHDR, "foob", testdata)
                .unwrap();
            assert_eq!(1, info.unknown_chunks(ChunkPosition::IHDR).count());
            info.create_chunk(ChunkPosition::IHDR, "foob", testdata)
                .unwrap();
            assert_eq!(2, info.unknown_chunks(ChunkPosition::IHDR).count());

            for _ in info.unknown_chunks(ChunkPosition::IDAT) {}
            let chunk =
                info.unknown_chunks(ChunkPosition::IHDR).next().unwrap();
            assert_eq!("foob".as_bytes(), chunk.name());
            assert!(chunk.is_type("foob"));
            assert!(!chunk.is_type("foobar"));
            assert!(!chunk.is_type("foo"));
            assert!(!chunk.is_type("FOOB"));
            assert!(chunk.check_crc());
            assert_eq!(testdata, chunk.data());
            info.get("foob").unwrap();
        }

        let raster: pix::Raster<pix::Rgba8> =
            pix::RasterBuilder::new().with_u8_buffer(1, 1, &[0u8, 0, 0, 0][..]);
        let img = state.encode(&raster).unwrap();
        let mut dec = State::new();
        dec.decode(img).unwrap();
        let chunk = dec
            .info_png()
            .unknown_chunks(ChunkPosition::IHDR)
            .next()
            .unwrap();
        assert_eq!("foob".as_bytes(), chunk.name());
        dec.info_png().get("foob").unwrap();
    }

    #[test]
    fn read_icc() {
        let mut s = State::new();
        let f = s.decode_file("tests/profile.png");
        f.unwrap();
        let icc = s.info_png().get("iCCP").unwrap();
        assert_eq!(275, icc.len());
        assert_eq!("ICC Pro".as_bytes(), &icc.data()[0..7]);

        let data = s.get_icc().unwrap();
        assert_eq!("appl".as_bytes(), &data[4..8]);
    }
}
