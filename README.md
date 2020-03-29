# PNG Pong

[![Build Status](https://api.travis-ci.org/RedAldaron/png-pong.svg?branch=master)](https://travis-ci.org/RedAldaron/png-pong)
[![Docs](https://docs.rs/png_pong/badge.svg)](https://docs.rs/png_pong)
[![crates.io](https://img.shields.io/crates/v/png-pong.svg)](https://crates.io/crates/png-pong)

> A pure Rust PNG/APNG encoder & decoder

This is a pure Rust PNG image decoder and encoder based on lodepng.
This crate allows easy reading and writing of PNG files without any
system dependencies.

### Why another PNG crate?
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

### Goals
- Forbid unsafe.
- APNG support as iterator.
- Fast.
- Compatible with pix / gift-style API.
- Load all PNG files crushed with pngcrush.
- Save crushed PNG files.
- Clean, well-documented, concise code.

### TODO
 - Implement APNG reading.
 - Implement Chunk reading (with all the different chunk structs).
 - RasterDecoder should wrap ChunkDecoder & RasterEncoder should wrap ChunkEncoder
 - Replace `ParseError` with Rust-style enum instead of having a C integer.
 - More test cases to test against.

## Table of Contents
- [Getting Started](#getting-started)
   - [Example](#example)
   - [API](#api)
   - [Features](#features)
- [Upgrade](#upgrade)
- [License](#license)
   - [Contribution](#contribution)

## Getting Started
Add the following to your `Cargo.toml`.

```toml
[dependencies]
png-pong = "0.2.0"
```

### Example
Say you want to read a PNG file into a raster:

```rust,no_run
let data = std::fs::read("graphic.png").expect("Failed to open PNG");
let data = std::io::Cursor::new(data);
let decoder = png_pong::FrameDecoder::<_, pix::SRgba8>::new(data);
let png_pong::Frame { raster, delay } = decoder
    .last()
    .expect("No frames in PNG")
    .expect("PNG parsing error");
```

Say you want to save a raster as a PNG file:

```rust,no_run
let raster = pix::RasterBuilder::new().with_pixels(1, 1, &[
    pix::SRgba8::new(0, 0, 0, 0)][..]
);
let mut out_data = Vec::new();
let mut encoder = png_pong::FrameEncoder::<_, pix::SRgba8>::new(
    &mut out_data
);
let frame = png_pong::Frame{ raster, delay: 0 };
encoder.encode(&frame).expect("Failed to add frame");
std::fs::write("graphic.png", out_data).expect("Failed to save image");
```

### API
API documentation can be found on [docs.rs](https://docs.rs/png-pong).

### Features
There is one optional feature "flate" which is enabled by default,
allowing png-pong to read compressed PNG files (which is most of them).
This pulls in the miniz\_oxide dependency.

## Upgrade
You can use the
[changelog](https://github.com/RedAldaron/png-pong/blob/master/CHANGELOG.md)
to facilitate upgrading this crate as a dependency.

## License
Licensed under either of
 - Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 - Zlib License, ([LICENSE-ZLIB](LICENSE-ZLIB) or
   https://opensource.org/licenses/Zlib)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Before contributing, check out the
[contribution guidelines](https://github.com/RedAldaron/png-pong/blob/master/CONTRIBUTING.md),
and, as always, make sure to always follow the
[code of conduct](https://github.com/RedAldaron/png-pong/blob/master/CODEOFCONDUCT.md).
