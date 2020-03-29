// PNG Pong
//
// Copyright © 2019-2020 Jeron Aldaron Lau
// Copyright © 2014-2017 Kornel Lesiński
// Copyright © 2005-2016 Lode Vandevenne
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Adler32                                                                  */
fn update_adler32(adler: u32, data: &[u8]) -> u32 {
    let mut s1 = adler & 65535;
    let mut s2 = (adler >> 16) & 65535;
    /*at least 5550 sums can be done before the sums overflow, saving a lot of module divisions*/
    for part in data.chunks(5550) {
        for &v in part {
            s1 += v as u32;
            s2 += s1;
        }
        s1 %= 65521;
        s2 %= 65521;
    }
    (s2 << 16) | s1
}

/*Return the adler32 of the bytes data[0..len-1]*/
pub(super) fn adler32(data: &[u8]) -> u32 {
    update_adler32(1, data)
}
