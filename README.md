# PNG Pong - A pure Rust PNG encoder & decoder
This is a pure Rust PNG image decoder and encoder based on lodepng.
This crate allows easy reading and writing of PNG files without any
system dependencies.

# Why another PNG crate?
These are the 3 Rust PNG encoder/decoder crates I know of:
- [png](https://crates.io/crates/png) - The one everyone uses, is very
  limited in which PNGs it can open.
- [lodepng](https://crates.io/crates/lodepng) - Lots of features, code
  is ported from C, therefore code is hard read & maintain, also uses
  slow implementation of deflate/inflate algorithm.
- [imagefmt](https://crates.io/crates/imagefmt) - Abandoned, just as
  limited as png, but with a lot less lines of code.

Originally I made the [aci_png](https://crates.io/crates/aci_png) based
on imagefmt, and intended to add more features.  That task seemed
possible at first, but just became daunting after a while.  That's why I
decided to take `lodepng` which has more features (and more low level
features) and clean up the code, upgrade to 2018 edition of Rust, depend
on the miniz\_oxide crate (because it can do it faster than lodepng) and
get rid of the libc dependency so it *actually* becomes pure Rust
(lodepng claims to be, but calls C's malloc and free).  I also decided
to model the API after the [gift](https://crates.io/crates/gift) crate,
so I'm using [pix](https://crates.io/crates/pix) instead of
[rgb](https://crates.io/crates/rgb).

## Goals
- Forbid unsafe.
- APNG support as iterator.
- Fast.
- Compatible with pix / gift-style API.
- Load all PNG files crushed with pngcrush.
- Save crushed PNG files.
- Clean, well-documented, concise code.

## Examples
- Say you want to read a PNG file into a raster:
```rust,no_run
let mut decoder_builder = png_pong::DecoderBuilder::new();
let data = std::fs::read("graphic.png").expect("Failed to open PNG");
let data = std::io::Cursor::new(data);
let decoder = decoder_builder.decode_rasters(data);
let (raster, _nanos) = decoder
    .last()
    .expect("No frames in PNG")
    .expect("PNG parsing error");
```

- Say you want to save a raster as a PNG file.
```rust,no_run
let raster = png_pong::RasterBuilder::new().with_pixels(1, 1, &[
    pix::Rgba8::with_alpha(
        pix::Ch8::new(0),
        pix::Ch8::new(0),
        pix::Ch8::new(0),
        pix::Ch8::new(0),
    )][..]
);
let mut out_data = Vec::new();
let mut encoder = png_pong::EncoderBuilder::new();
let mut encoder = encoder.encode_rasters(&mut out_data);
encoder.add_frame(&raster, 0).expect("Failed to add frame");
std::fs::write("graphic.png", out_data).expect("Failed to save image");
```

## TODO
- Implement APNG reading.
- Implement Chunk reading (with all the different chunk structs).
- RasterDecoder should wrap ChunkDecoder & RasterEncoder should wrap ChunkEncoder
- Replace `ParseError` with Rust-style enum instead of having a C integer.
- More test cases to test against.
