// png-pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
// Copyright © 2014-2017 Kornel Lesiński
// Copyright © 2005-2016 Lode Vandevenne
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::Format;
use pix::Raster;

/// A Frame
pub struct Frame<F: Format> {
    /// Raster associated with this frame.
    pub raster: Raster<F>,
    /// TODO: Delay associated with this frame.
    pub delay: u32,
}

impl<F: Format> std::fmt::Debug for Frame<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.delay)
    }
}
