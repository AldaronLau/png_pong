use std::{collections::HashMap, io::Read, iter::Peekable};

use pix::{Palette, Raster};

use crate::{
    chunk::{
        Background, Chunk, ColorType, ImageHeader, Palette as PaletteChunk,
        Physical, Time, Transparency,
    },
    consts,
    decode::{Chunks, Error as DecoderError},
    zlib, PngRaster, Step,
};

mod unfilter;

#[derive(Debug)]
struct TextEntry {
    #[allow(dead_code)] // FIXME
    text: String,
    #[allow(dead_code)] // FIXME
    langtag: Option<String>,
    #[allow(dead_code)] // FIXME
    transkey: Option<String>,
}

/// Iterator over `Step`s for PNG files.
#[derive(Debug)]
pub struct Steps<R: Read> {
    decoder: Peekable<Chunks<R>>,
    // FIXME: This is a workaround for not supporting APNG yet.
    #[allow(dead_code)]
    has_decoded: bool,
    // None if haven't decoded a frame yet.
    header: Option<ImageHeader>,
    // Is the file an APNG animation?
    #[allow(dead_code)]
    is_animation: bool,
    // Is IDAT part of the animation?
    #[allow(dead_code)]
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
    // True if after palette chunk found
    reject_pal: bool,
}

impl<R: Read> Steps<R> {
    /// Create a new decoder.
    pub(crate) fn new(chunks: Chunks<R>) -> Self {
        let decoder = chunks.peekable();

        Self {
            decoder,
            has_decoded: false,
            header: None,
            idat_anim: false,
            is_animation: false,
            palette: None,
            transparency: None,
            background: None,
            physical: None,
            text: HashMap::new(),
            time: None,
            reject_pal: false,
        }
    }
}

impl<R> Iterator for Steps<R>
where
    R: Read,
{
    type Item = Result<Step, DecoderError>;

    fn next(&mut self) -> Option<Self::Item> {
        // First frame
        if self.header.is_none() {
            // First chunk must be IHDR
            self.header = match self.decoder.next().ok_or(DecoderError::Empty) {
                Ok(Ok(Chunk::ImageHeader(header))) => Some(header),
                Ok(Ok(_chunk)) => return Some(Err(DecoderError::ChunkOrder)),
                Ok(Err(e)) => return Some(Err(e)),
                Err(e) => return Some(Err(e)),
            };

            // Go through chunks before IDAT
            while {
                match self.decoder.peek() {
                    Some(Ok(chunk)) => !chunk.is_idat(),
                    Some(Err(DecoderError::UnknownChunkType(_))) => true,
                    Some(Err(e)) => return Some(Err(e.clone())),
                    None => return Some(Err(DecoderError::NoImageData)),
                }
            } {
                use Chunk::*;
                // Won't panic
                let chunk = if let Ok(chunk) = self.decoder.next().unwrap() {
                    chunk
                } else {
                    continue; // Skip unknown chunks
                };
                match chunk {
                    Palette(chunk) => {
                        if self.reject_pal {
                            return Some(Err(DecoderError::ChunkOrder));
                        }
                        if self.palette.is_some() {
                            return Some(Err(DecoderError::Multiple(
                                consts::PALETTE,
                            )));
                        }
                        self.palette = Some(chunk)
                    }
                    Background(chunk) => {
                        self.reject_pal = true;
                        if self.background.is_some() {
                            return Some(Err(DecoderError::Multiple(
                                consts::BACKGROUND,
                            )));
                        }
                        self.background = Some(chunk);
                    }
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
                    Physical(chunk) => {
                        if self.physical.is_some() {
                            return Some(Err(DecoderError::Multiple(
                                consts::PHYSICAL,
                            )));
                        }
                        self.physical = Some(chunk);
                    }
                    Time(chunk) => {
                        if self.time.is_some() {
                            return Some(Err(DecoderError::Multiple(
                                consts::TIME,
                            )));
                        }
                        self.time = Some(chunk);
                    }
                    Transparency(chunk) => {
                        self.reject_pal = true;
                        if self.transparency.is_some() {
                            return Some(Err(DecoderError::Multiple(
                                consts::TRANSPARENCY,
                            )));
                        }
                        self.transparency = Some(chunk);
                    }
                    ImageHeader(_) => {
                        return Some(Err(DecoderError::ChunkOrder))
                    }
                    ImageEnd(_) => return Some(Err(DecoderError::NoImageData)),
                    ImageData(_) => unreachable!(),
                    Unknown(_) => continue, // Skip unknown chunks
                }
            }
        }

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
            self.header.as_ref().unwrap(),
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
                Unknown(unknown) => {
                    return Some(Err(DecoderError::UnknownChunkType(
                        unknown.name,
                    )))
                }
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
        header,
    )?;

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
            let palette_alpha = match transparency {
                None => Vec::new(),
                Some(Transparency::Palette(p)) => p.to_vec(),
                _ => unreachable!(),
            };
            let mut palette = Palette::new(palette_slice.len());
            for (i, color) in palette_slice.iter().enumerate() {
                let j = palette.set_entry(*color).unwrap();
                debug_assert_eq!(i, j);
            }
            debug_assert_eq!(palette_slice.len(), palette.len());
            PngRaster::Palette(
                Raster::with_u8_buffer(width, height, buf),
                Box::new(palette),
                palette_alpha,
            )
        }
        (ct, bd) => return Err(DecoderError::ColorMode(ct, bd)),
    })
}
