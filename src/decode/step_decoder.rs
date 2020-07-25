// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::{collections::HashMap, io::Read, iter::Peekable};

use pix::{Palette, Raster};

use crate::{
    chunk::{
        Background, Chunk, ColorType, ImageHeader, Palette as PaletteChunk,
        Physical, Time, Transparency,
    },
    consts,
    decode::ChunkDecoder,
    decode::Error as DecoderError,
    zlib, PngRaster, Step,
};

mod unfilter;

#[derive(Debug)]
struct TextEntry {
    text: String,
    langtag: Option<String>,
    transkey: Option<String>,
}

/// Iterator over `Step`s for PNG files.
#[derive(Debug)]
pub struct StepDecoder<R: Read> {
    decoder: Peekable<ChunkDecoder<R>>,
    // FIXME: This is a workaround for not supporting APNG yet.
    has_decoded: bool,
    //
    header: ImageHeader,
    // Is the file an APNG animation?
    is_animation: bool,
    // Is IDAT part of the animation?
    idat_anim: bool,
    //
    palette: Option<PaletteChunk>,
    //
    transparency: Option<Transparency>,
    //
    background: Option<Background>,
    //
    text: HashMap<String, TextEntry>,
    //
    physical: Option<Physical>,
    //
    time: Option<Time>,
}

impl<R: Read> StepDecoder<R> {
    /// Create a new decoder.
    pub fn new(r: R) -> Result<Self, DecoderError> {
        let mut decoder = ChunkDecoder::new(r)?.peekable();
        // First chunk must be IHDR
        let header = match decoder.next().ok_or(DecoderError::Empty)?? {
            Chunk::ImageHeader(header) => header,
            _chunk => return Err(DecoderError::ChunkOrder),
        };
        let mut palette = None;
        let mut background = None;
        let mut physical = None;
        let mut reject_pal = false;
        let mut text = HashMap::new();
        let mut time = None;
        let mut transparency = None;
        // Go through chunks before IDAT
        while {
            match decoder.peek() {
                Some(Ok(chunk)) => !chunk.is_idat(),
                Some(Err(DecoderError::UnknownChunkType(_))) => true,
                Some(Err(e)) => return Err(e.clone()),
                None => return Err(DecoderError::NoImageData),
            }
        } {
            use Chunk::*;
            // Won't panic
            let chunk = if let Ok(chunk) = decoder.next().unwrap() {
                chunk
            } else {
                continue; // Skip unknown chunks
            };
            match chunk {
                Palette(chunk) => {
                    if reject_pal {
                        return Err(DecoderError::ChunkOrder);
                    }
                    if palette.is_some() {
                        return Err(DecoderError::Multiple(consts::PALETTE));
                    }
                    palette = Some(chunk)
                }
                Background(chunk) => {
                    reject_pal = true;
                    if background.is_some() {
                        return Err(DecoderError::Multiple(consts::BACKGROUND));
                    }
                    background = Some(chunk);
                }
                InternationalText(chunk) => {
                    text.insert(
                        chunk.key,
                        TextEntry {
                            text: chunk.val,
                            langtag: Some(chunk.langtag),
                            transkey: Some(chunk.transkey),
                        },
                    );
                }
                CompressedText(chunk) => {
                    text.insert(
                        chunk.key,
                        TextEntry {
                            text: chunk.val,
                            langtag: None,
                            transkey: None,
                        },
                    );
                }
                Text(chunk) => {
                    text.insert(
                        chunk.key,
                        TextEntry {
                            text: chunk.val,
                            langtag: None,
                            transkey: None,
                        },
                    );
                }
                Physical(chunk) => {
                    if physical.is_some() {
                        return Err(DecoderError::Multiple(consts::PHYSICAL));
                    }
                    physical = Some(chunk);
                }
                Time(chunk) => {
                    if time.is_some() {
                        return Err(DecoderError::Multiple(consts::TIME));
                    }
                    time = Some(chunk);
                }
                Transparency(chunk) => {
                    reject_pal = true;
                    if transparency.is_some() {
                        return Err(DecoderError::Multiple(
                            consts::TRANSPARENCY,
                        ));
                    }
                    transparency = Some(chunk);
                }
                ImageHeader(_) => return Err(DecoderError::ChunkOrder),
                ImageEnd(_) => return Err(DecoderError::NoImageData),
                ImageData(_) => unreachable!(),
            }
        }

        Ok(Self {
            decoder,
            has_decoded: false,
            header,
            idat_anim: false,
            is_animation: false,
            palette,
            transparency,
            background,
            physical,
            text,
            time,
        })
    }
}

impl<R> Iterator for StepDecoder<R>
where
    R: Read,
{
    type Item = Result<Step, DecoderError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check for ImageEnd
        if let Some(Ok(chunk)) = self.decoder.peek() {
            if chunk.is_iend() {
                if let Err(e) = self.decoder.next().unwrap() {
                    return Some(Err(e));
                }
                if self.decoder.next().is_some() {
                    return Some(Err(DecoderError::TrailingChunk));
                }
                return None;
            }
        }

        // Image data for consecutive IDAT chunks.
        let mut idat = Vec::new();

        // Go through until the last IDAT or fdAT chunk.
        while {
            let chunk = match self.decoder.peek() {
                Some(Ok(chunk)) => chunk,
                Some(Err(e)) => return Some(Err(e.clone())),
                None => return Some(Err(DecoderError::NoImageData)),
            };
            chunk.is_idat()
        } {
            match self.decoder.next().unwrap() {
                Ok(Chunk::ImageData(data)) => idat.extend(data.data),
                Ok(_) => unreachable!(),
                Err(e) => return Some(Err(e)),
            }
        }

        let raster = match decode(
            idat.as_slice(),
            &self.header,
            self.palette.as_ref(),
            self.transparency.as_ref(),
        ) {
            Ok(raster) => raster,
            Err(e) => return Some(Err(e)),
        };

        // Check for non-required chunks up until the next IDAT or fdAT chunk or
        // end
        while {
            let chunk = match self.decoder.peek() {
                Some(Ok(chunk)) => chunk,
                Some(Err(e)) => return Some(Err(e.clone())),
                None => return Some(Err(DecoderError::NoImageData)),
            };
            !chunk.is_idat() && !chunk.is_iend()
        } {
            use Chunk::*;
            match self.decoder.next().unwrap().unwrap() {
                // won't panic
                InternationalText(chunk) => {
                    self.text.insert(
                        chunk.key,
                        TextEntry {
                            text: chunk.val,
                            langtag: Some(chunk.langtag),
                            transkey: Some(chunk.transkey),
                        },
                    );
                }
                CompressedText(chunk) => {
                    self.text.insert(
                        chunk.key,
                        TextEntry {
                            text: chunk.val,
                            langtag: None,
                            transkey: None,
                        },
                    );
                }
                Text(chunk) => {
                    self.text.insert(
                        chunk.key,
                        TextEntry {
                            text: chunk.val,
                            langtag: None,
                            transkey: None,
                        },
                    );
                }
                Time(chunk) => {
                    if self.time.is_some() {
                        return Some(Err(DecoderError::Multiple(consts::TIME)));
                    }
                    self.time = Some(chunk);
                }
                ImageHeader(_) => return Some(Err(DecoderError::ChunkOrder)),
                Palette(_) => return Some(Err(DecoderError::ChunkOrder)),
                Background(_) => return Some(Err(DecoderError::ChunkOrder)),
                Physical(_) => return Some(Err(DecoderError::ChunkOrder)),
                Transparency(_) => return Some(Err(DecoderError::ChunkOrder)),
                ImageData(_) => unreachable!(),
                ImageEnd(_) => unreachable!(),
            }
        }

        Some(Ok(Step { raster, delay: 0 }))
    }
}

/// Decode one `Step` from header and compressed pixel data.
pub(crate) fn decode(
    buffer: &[u8],
    header: &ImageHeader,
    palette: Option<&PaletteChunk>,
    transparency: Option<&Transparency>,
) -> Result<PngRaster, DecoderError> {
    // Decompress and unfilter pixel data.
    let mut scanlines = zlib::decompress(buffer)?;
    let mut buf = vec![0; header.raw_size()];
    unfilter::postprocess_scanlines(
        &mut buf,
        &mut scanlines,
        header.width,
        header.height,
        &header,
    )?;

    /*let input = input.as_ref();
    let (buf, header, palette, transparency) =
        decode_generic(true, input)?;*/
    let width = header.width;
    let height = header.height;
    let color_type = header.color_type;
    let bit_depth = header.bit_depth;

    Ok(match (color_type, bit_depth) {
        (ColorType::Grey, 8) => {
            PngRaster::Gray8(Raster::with_u8_buffer(width, height, buf))
        }
        (ColorType::GreyAlpha, 8) => {
            PngRaster::Graya8(Raster::with_u8_buffer(width, height, buf))
        }
        (ColorType::Rgb, 8) => {
            PngRaster::Rgb8(Raster::with_u8_buffer(width, height, buf))
        }
        (ColorType::Rgba, 8) => {
            PngRaster::Rgba8(Raster::with_u8_buffer(width, height, buf))
        }
        (ColorType::Grey, 16) => {
            let mut raster = Raster::with_clear(width, height);
            for (i, v) in raster.as_u8_slice_mut().iter_mut().enumerate() {
                *v = buf[i];
            }
            PngRaster::Gray16(raster)
        }
        (ColorType::GreyAlpha, 16) => {
            let mut raster = Raster::with_clear(width, height);
            for (i, v) in raster.as_u8_slice_mut().iter_mut().enumerate() {
                *v = buf[i];
            }
            PngRaster::Graya16(raster)
        }
        (ColorType::Rgb, 16) => {
            let mut raster = Raster::with_clear(width, height);
            for (i, v) in raster.as_u8_slice_mut().iter_mut().enumerate() {
                *v = buf[i];
            }
            PngRaster::Rgb16(raster)
        }
        (ColorType::Rgba, 16) => {
            let mut raster = Raster::with_clear(width, height);
            for (i, v) in raster.as_u8_slice_mut().iter_mut().enumerate() {
                *v = buf[i];
            }
            PngRaster::Rgba16(raster)
        }
        (ColorType::Palette, 8) => {
            let palette_slice = palette.as_ref().unwrap().palette.as_slice();
            let palette_alpha = match transparency.unwrap() {
                Transparency::Palette(p) => p,
                _ => unreachable!(),
            };
            let mut palette = Palette::new(palette_slice.len());
            for (i, color) in palette_slice.iter().enumerate() {
                debug_assert_eq!(i, palette.set_entry(*color).unwrap());
            }
            debug_assert_eq!(palette_slice.len(), palette.len());
            PngRaster::Palette(
                Raster::with_u8_buffer(width, height, buf),
                Box::new(palette),
                palette_alpha.to_vec(),
            )
        }
        (ct, bd) => return Err(DecoderError::ColorMode(ct, bd)),
    })
}
