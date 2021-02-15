# ![PNG Pong](https://raw.githubusercontent.com/AldaronLau/png_pong/master/res/icon.png)

#### A pure Rust PNG/APNG encoder & decoder

[![tests](https://github.com/AldaronLau/png_pong/workflows/tests/badge.svg)](https://github.com/AldaronLau/png_pong/actions?query=workflow%3Atests)
[![docs](https://docs.rs/png_pong/badge.svg)](https://docs.rs/png_pong)
[![crates.io](https://img.shields.io/crates/v/png_pong.svg)](https://crates.io/crates/png_pong)

This is a pure Rust PNG image decoder and encoder taking inspiration from
lodepng.  This crate allows easy reading and writing of PNG files without any
system dependencies.

### Why another PNG crate?
These are the 4 Rust PNG encoder/decoder crates I know of:
- [png](https://crates.io/crates/png) - The one everyone uses (used to be able
  to load less pngs than png_pong and slower, but has caught up).
- [lodepng](https://crates.io/crates/lodepng) - Loads all the PNGs, code
  is ported from C, therefore code is hard read & maintain, also uses
  slow implementation of deflate/inflate algorithm.
- [imagefmt](https://crates.io/crates/imagefmt) - Abandoned, and
  limited in what files it can open, but with a lot less lines of code.
- [imagine](https://crates.io/crates/imagine) - PNG decoding only.

Originally I made the [aci_png](https://crates.io/crates/aci_png) based
on imagefmt, and intended to add more features.  At the time, I didn't want to
write a PNG encoder/decoder from scratch so I decided to take `lodepng` which
has more features (and more low level features) and clean up the code, upgrade
to 2018 edition of Rust, depend on the miniz\_oxide crate (because it can
decompress faster than lodepng's INFLATE implementation) and get rid of the libc
dependency so it *actually* becomes pure Rust (lodepng claims to be, but calls
C's malloc and free).  Then, I rewrote the entire library, based on
[gift](https://crates.io/crates/gift) and [pix](https://crates.io/crates/pix).

### Goals
 - Forbid unsafe.
 - APNG support as iterator.
 - Fast.
 - Compatible with pix / gift-style API.
 - Load all PNG files crushed with pngcrush.
 - Save crushed PNG files.
 - Clean, well-documented, concise code.
 - Implement all completed, non-deprecated chunks in the
   [PNG 1.2 Specification](http://www.libpng.org/pub/png/spec/1.2/PNG-Contents.html),
   including the
   [PNG 1.2 Extensions](https://pmt.sourceforge.io/specs/pngext-1.2.0-pdg-h20.html)
   and the
   [APNG Specification](https://wiki.mozilla.org/APNG_Specification)

### TODO
 - Implement APNG reading.
 - Implement Chunk reading (with all the different chunk structs).
 - StepDecoder should wrap StepDecoder & RasterEncoder should wrap ChunkEncoder
 - Replace `ParseError` with Rust-style enum instead of having a C integer.
 - More test cases to test against.

### Benchmarks And Comparisons
Using Rust 1.45.0, criterion and 4 different PNG sizes with PNGs from
"./tests/png/" (units are: us / microseconds).  I stopped anything that was
predicted to take longer than a half hour with criterion with the message
"TIMEOUT".

- sRGB 1x1: Uses `tests/png/profile.png`
- sRGBA 1x1: Uses `tests/png/test.png`
- sRGB 64x64: Uses `test/png/4.png`
- sRGBA 64x64: Uses `tests/png/res.png`
- sRGB 256x256: `tests/png/PngSuite.png`
- sRGBA 256x256: Uses `tests/png/icon.png`
- sRGB 4096x4096: `tests/png/plopgrizzly.png`
- sRGBA 4096x4096: Uses `tests/png/noise.png`

#### Decoder
| Library    | sRGB 1x1 | sRGBA 1x1 | sRGB 64x64 | sRGBA 64x64 | sRGB 256x256 | sRGBA 256x256 | sRGB 4096x4096 | sRGBA 4096x4096 |
|------------|----------|-----------|------------|-------------|--------------|---------------|----------------|-----------------|
| png_pong   | 7.7904   | 4.1947    | 82.783     | 101.75      | 864.58       | 905.51        | 174,040        | 542,570         |
| png        | 10.656   | 6.5267    | 52.879     | 70.930      | 634.74       | 686.75        | 119,790        | 300,980         |
| lodepng    | 9.0110   | 8.5484    | 193.79     | 200.67      | 856.78       | 1,280.4       | 196,740        | 1,722,800       |
| imagefmt   | 4.0710   | 3.8689    | 63.258     | 69.192      | 491.58       | 637.12        | 67,663         | 464,730         |
| imagine    | 2.8407   | 0.52495   | 3,135.8    | 1,938.7     | 1,655.4      | 9,473.0       | 404,520        | TIMEOUT         |
| aci_png    | 3.9223   | 4.1440    | 212.54     | 177.63      | 1,395.7      | 1,674.3       | 373,510        | 1,242,000       |
| libpng-sys | 3.2617   | 0.43611   | 1.8694     | 0.58886     | 24.782       | 4.1214        | 17,539         | 17,259          |

#### Encoder
| Library    | sRGB 1x1 | sRGBA 1x1 | sRGB 64x64 | sRGBA 64x64 | sRGB 256x256 | sRGBA 256x256 | sRGB 4096x4096 | sRGBA 4096x4096 |
|------------|----------|-----------|------------|-------------|--------------|---------------|----------------|-----------------|
| png_pong   | 42.012   | 9.9705    | 1,038.2    | 721.43      | 2,575.2      | 5,105.4       | 579,200        | 3,201,900       |
| png        | 25.448   | 9.1111    | 192.52     | 190.61      | 868.28       | 1,432.2       | 184,340        | 1,384,400       |
| lodepng    | 12.241   | 11.915    | 2,005.2    | 4,361.0     | 24,595       | 162,510       | TIMEOUT        | TIMEOUT         |
| imagefmt   | 8.1248   | 9.7751    | 151.89     | 140.72      | 819.41       | 1,483.4       | 214,010        | 770,080         |
| imagine    | ---      | ---       | ---        | ---         | ---          | ---           | ---            | ---             |
| aci_png    | FAILS    | 28.228    | FAILS      | 245.12      | FAILS        | 2,167.0       | FAILS          | 1,823,400       |                |                 |
| libpng-sys | 3.0473   | 0.038876  | 3.0797     | 0.039217    | 2.7762       | 0.039250      | 3.7263         | 0.039266        |

## Table of Contents
- [API](#api)
- [Features](#features)
- [Upgrade](#upgrade)
- [License](#license)
   - [Contribution](#contribution)

## API
API documentation can be found on [docs.rs](https://docs.rs/png_pong).

## Features
There are no optional features.

## Upgrade
You can use the
[changelog](https://github.com/AldaronLau/png_pong/blob/master/CHANGELOG.md)
to facilitate upgrading this crate as a dependency.

## License
Licensed under either of
 - Apache License, Version 2.0,
   ([LICENSE-APACHE](https://github.com/AldaronLau/png_pong/blob/master/LICENSE-APACHE)
   or https://www.apache.org/licenses/LICENSE-2.0)
 - Zlib License,
   ([LICENSE-ZLIB](https://github.com/AldaronLau/png_pong/blob/master/LICENSE-ZLIB)
   or https://opensource.org/licenses/Zlib)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Before contributing, check out the
[contribution guidelines](https://github.com/AldaronLau/png_pong/blob/master/CONTRIBUTING.md),
and, as always, make sure to always follow the
[code of conduct](https://github.com/AldaronLau/png_pong/blob/master/CODE_OF_CONDUCT.md).
