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
 - StepDecoder should wrap StepDecoder, RasterEncoder should wrap ChunkEncoder
 - More test cases to test against.

### Benchmarks And Comparisons
Using Rust 1.52.1, criterion and 4 different PNG sizes with PNGs from
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

#### Encoder
| Library    | sRGB 1x1 | sRGBA 1x1 | sRGB 64x64 | sRGBA 64x64 | sRGB 256x256 | sRGBA 256x256 | sRGB 4096x4096 | sRGBA 4096x4096 |
|------------|----------|-----------|------------|-------------|--------------|---------------|----------------|-----------------|
| png_pong   | 41.956   | 8.2661    | 1\_025.7   | 700.80      | 2\_646.1     | 5\_061.5      | 587\_320       | 3\_587\_100     |
| png        | 29.538   | 9.4122    | 213.49     | 203.02      | 944.99       | 1\_534.3      | 201\_680       | 1\_535\_300     |
| lodepng    | 59.989   | 10.700    | 1\_399.3   | 982.63      | 3\_391.3     | 6\_664.7      | 831\_190       | 3\_394\_900     |
| imagefmt   | 8.7942   | 8.7399    | 211.01     | 177.82      | 901.22       | 1\_569.4      | 218\_550       | 1\_285\_700     |
| imagine    | ---      | ---       | ---        | ---         | ---          | ---           | ---            | ---             |
| aci_png    | FAILS    | 30.987    | FAILS      | 289.24      | FAILS        | 2\_298.1      | FAILS          | 2\_135\_400     |
| libpng-sys | 6.8443   | 2.9461    | 1\_613.5   | 769.70      | 2\_261.1     | 4\_745.2      | 520\_770       | 2\_926\_900     |

#### Decoder
| Library    | sRGB 1x1 | sRGBA 1x1 | sRGB 64x64 | sRGBA 64x64 | sRGB 256x256 | sRGBA 256x256 | sRGB 4096x4096 | sRGBA 4096x4096 |
|------------|----------|-----------|------------|-------------|--------------|---------------|----------------|-----------------|
| png_pong   | 7.7520   | 3.9459    | 77.981     | 99.384      | 752.95       | 901.98        | 178\_880       | 570\_200        |
| png        | 8.1195   | 6.6107    | 54.834     | 71.873      | 643.09       | 686.29        | 128\_000       | 355\_080        |
| lodepng    | 5.8958   | 5.4527    | 77.050     | 97.762      | 882.83       | 982.76        | 230\_570       | 563\_210        |
| imagefmt   | 4.2864   | 4.8706    | 74.715     | 82.026      | 566.86       | 758.27        | 69\_465        | 545\_060        |
| imagine    | 2.8809   | 0.44822   | 3\_202.3   | 2\_266.4    | 2\_056.1     | 10\_753       | 442\_750       | 27\_944\_000    |
| aci_png    | 5.0243   | 4.3516    | 201.29     | 174.30      | 1\_500.6     | 1\_689.8      | 398\_340       | 1\_323\_600     |
| libpng-sys | 3.6011   | 0.48747   | 1.8175     | 0.67344     | 25.809       | 4.4175        | 19\_400        | 18\_262         |

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
Copyright © 2019-2023 The PNG Pong Crate Contributor(s)

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
