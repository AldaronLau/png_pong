//! Compression algorithms

use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec};

use crate::decode::Error;

// FIXME: Streaming API
pub(crate) fn decompress(inp: &[u8]) -> Result<Vec<u8>, Error> {
    if inp.len() < 2 {
        return Err(Error::ZlibTooSmall);
    }
    /* read information from zlib header */
    if (inp[0] as u32 * 256 + inp[1] as u32) % 31 != 0 {
        /* error: 256 * in[0] + in[1] must be a multiple of 31, the FCHECK
         * value is supposed to be made that way */
        return Err(Error::ZlibHeader);
    }
    let cm = inp[0] as u32 & 15;
    let cinfo = ((inp[0] as u32) >> 4) & 15;
    let fdict = ((inp[1] as u32) >> 5) & 1;
    if cm != 8 || cinfo > 7 {
        /* error: only compression method 8: inflate with sliding window of
         * 32k is supported by the PNG spec */
        return Err(Error::CompressionMethod);
    }
    if fdict != 0 {
        /*error: the specification of PNG says about the zlib stream:
        "The additional flags shall not specify a preset dictionary."*/
        return Err(Error::PresetDict);
    }

    let out = match decompress_to_vec(&inp[2..(inp.len() - 4)]) {
        Ok(rtn) => rtn,
        Err(e) => {
            return Err(Error::Inflate(e.status));
        }
    };

    let adler32_val = u32::from_be_bytes([
        inp[inp.len() - 4],
        inp[inp.len() - 3],
        inp[inp.len() - 2],
        inp[inp.len() - 1],
    ]);
    let checksum = adler32(&out);
    if checksum != adler32_val {
        return Err(Error::AdlerChecksum);
    }

    Ok(out)
}

// FIXME: Streaming API
pub(crate) fn compress(outv: &mut Vec<u8>, inp: &[u8], level: u8) {
    /*initially, *out must be NULL and outsize 0, if you just give some random *out
    that's pointing to a non allocated buffer, this'll crash*/
    /* zlib data: 1 byte CMF (cm+cinfo), 1 byte FLG, deflate data, 4 byte
     * adler32_val checksum of the Decompressed data */
    let cmf = 120;
    /* 0b01111000: CM 8, cinfo 7. With cinfo 7, any window size up to 32768
     * can be used. */
    let flevel = 0;
    let fdict = 0;
    let mut cmfflg = 256 * cmf + fdict * 32 + flevel * 64;
    let fcheck = 31 - cmfflg % 31;
    cmfflg += fcheck;
    /* Vec<u8>-controlled version of the output buffer, for dynamic array */
    outv.push((cmfflg >> 8) as u8);
    outv.push((cmfflg & 255) as u8);
    let deflated = compress_to_vec(inp, level);
    let adler32_val = adler32(inp);
    outv.extend_from_slice(&deflated);
    outv.extend(adler32_val.to_be_bytes().iter());
}

/// Return the Adler32 of the bytes data[0..len-1]
fn adler32(data: &[u8]) -> u32 {
    let mut adler = simd_adler32::Adler32::new();
    adler.write(data);
    adler.finish()
}
