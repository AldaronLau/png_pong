# Changelog
All notable changes to PNG Pong will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://jeronlau.tk/semver/).

## 0.9.0 - Unreleased
### Changed
 - Updated dependencies
 - Result type aliases are now more flexible
 - Upgraded to the 2021 edition
 - Depend on `simd-adler32` crate for speed improvements

### Fixed
 - Warnings, clippy lints

## 0.8.2 - 2021-05-18
### Fixed
 - Palette PNGs returning empty palette when compiled in release mode.

## 0.8.1 - 2021-05-16
### Fixed
 - Not being able to load palette PNGs without a transparency chunk.

## 0.8.0 - 2021-02-14
Thanks to [wezm](https://github.com/wezm) for adding these features!

### Added
 - `chunk::Unknown`
 - Implemented `std::error::Error` on `encode::Error`
 - Implemented `std::error::Error` on `decode::Error`

## 0.7.0 - 2020-09-19
### Added
 - Sealed trait: `AsRaster`

### Changed
 - `StepEnc.still()` now takes either a reference to a PngRaster or a Raster

## 0.6.0 - 2020-07-26
### Added
- `chunk::ColorType` and `PngRaster` for reading PNGs without conversion
- Lots of chunks to the Chunk API
  - `CompressedText`
  - `ImageData`
  - `ImageEnd`
  - `ImageHeader`
  - `Palette`
  - `Physical`
  - `Time`
  - `Background`
  - `Transparency`
- `encode::Result`
- `decode::Result`
- `Decoder` - A builder for decoder types
- `Encoder` - A builder for encoder types

### Changed
- Renamed `EncodeError` -> `encode::Error`
- Renamed `DecodeError` -> `decode::Error`
- Renamed `chunk::TextChunk` -> `chunk::Text`
- Renamed `chunk::ITextChunk` -> `InternationalText`
- Renamed `Frame` -> `Step`
- Renamed `ChunkDecoder` -> `decode::Chunks`
- Renamed `ChunkEncoder` -> `encode::ChunkEnc`
- Renamed `FrameDecoder` -> `decode::Steps`
- Renamed `FrameEncoder` -> `encode::StepEnc`

### Removed
- `Format` trait
- `ParseError`, the very lame integer error.

### Fixed
- Chunk APIs not working

## 0.5.0 - 2020-05-05
### Changed
- Update to pix 0.13

## 0.4.0 - 2020-04-24
### Changed
- Update to pix 0.12

## 0.3.0 - 2020-04-11
### Changed
- Update to pix 0.11

## 0.2.2 - 2020-03-29
### Fixed
- Docs not building at all

## 0.2.1 - 2020-03-29
### Fixed
- Not all docs showing up on docs.rs

## 0.2.0 - 2020-03-29
### Changed
- Updated pix to 0.10
- Made `ColorType` a public item in the crate

## 0.1.0 - 2020-01-01
### Added
- `Frame` struct
- `Format` trait for pixel formats that can be saved as PNG

### Changed
- Replace `RasterDecoder` and `RasterEncoder` with `FrameEncoder` and
  FrameDecoder

### Removed
- Prelude module
- Re-exports from pix crate

## 0.0.2 - 2019-08-03
### Changed
- Use miniz\_oxide instead of deflate & inflate crates.

## 0.0.1 - 2019-07-24
### Added
- Support for reading writing PNGs.
