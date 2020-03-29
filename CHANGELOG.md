# Changelog
All notable changes to PNG Pong will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://jeronlau.tk/semver/).

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
