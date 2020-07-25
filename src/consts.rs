// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

// Magic bytes to start a PNG file.
pub(super) const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

// Chunk Identifiers
pub(super) const IMAGE_HEADER: [u8; 4] = *b"IHDR";
pub(super) const IMAGE_DATA: [u8; 4] = *b"IDAT";
pub(super) const BACKGROUND: [u8; 4] = *b"bKGD";
pub(super) const TRANSPARENCY: [u8; 4] = *b"tRNS";
pub(super) const IMAGE_END: [u8; 4] = *b"IEND";
pub(super) const PALETTE: [u8; 4] = *b"PLTE";
pub(super) const ITEXT: [u8; 4] = *b"iTXt";
pub(super) const PHYSICAL: [u8; 4] = *b"pHYs";
pub(super) const TIME: [u8; 4] = *b"tIME";
pub(super) const ZTEXT: [u8; 4] = *b"zTXt";
pub(super) const TEXT: [u8; 4] = *b"tEXt";
