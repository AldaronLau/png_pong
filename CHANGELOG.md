# Changelog

## Unreleased

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
