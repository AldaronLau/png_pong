//! Zlib     
use miniz_oxide::inflate::decompress_to_vec;

use super::*;

pub(crate) fn lodepng_zlib_decompress(
    inp: &[u8],
    settings: &DecompressSettings,
) -> Result<Vec<u8>, Error> {
    if inp.len() < 2 {
        return Err(Error(53));
    }
    /*read information from zlib header*/
    if (inp[0] as u32 * 256 + inp[1] as u32) % 31 != 0 {
        /*error: 256 * in[0] + in[1] must be a multiple of 31, the FCHECK value is supposed to be made that way*/
        return Err(Error(24));
    }
    let cm = inp[0] as u32 & 15;
    let cinfo = ((inp[0] as u32) >> 4) & 15;
    let fdict = ((inp[1] as u32) >> 5) & 1;
    if cm != 8 || cinfo > 7 {
        /*error: only compression method 8: inflate with sliding window of 32k is supported by the PNG spec*/
        return Err(Error(25));
    }
    if fdict != 0 {
        /*error: the specification of PNG says about the zlib stream:
        "The additional flags shall not specify a preset dictionary."*/
        return Err(Error(26));
    }

    let out = match decompress_to_vec(&inp[2..]) {
        Ok(rtn) => rtn,
        Err(e) => {
            eprintln!("Inflate Failure: {:?}", e);
            return Err(Error(52));
        }
    };

    if (!cfg!(fuzzing)) && settings.check_adler32 {
        let adler32_val = lodepng_read32bit_int(&inp[(inp.len() - 4)..]);
        let checksum = adler32(&out);
        /*error, adler checksum not correct, data must be corrupted*/
        if checksum != adler32_val {
            return Err(Error(58));
        };
    }
    Ok(out)
}

pub(crate) fn zlib_decompress(
    inp: &[u8],
    settings: &DecompressSettings,
) -> Result<Vec<u8>, Error> {
    lodepng_zlib_decompress(inp, settings)
}

pub(crate) fn lodepng_zlib_compress(
    outv: &mut Vec<u8>,
    inp: &[u8],
    settings: &CompressSettings,
) -> Result<(), Error> {
    /*initially, *out must be NULL and outsize 0, if you just give some random *out
    that's pointing to a non allocated buffer, this'll crash*/
    /*zlib data: 1 byte CMF (cm+cinfo), 1 byte FLG, deflate data, 4 byte adler32_val checksum of the Decompressed data*/
    let cmf = 120;
    /*0b01111000: CM 8, cinfo 7. With cinfo 7, any window size up to 32768 can be used.*/
    let flevel = 0;
    let fdict = 0;
    let mut cmfflg = 256 * cmf + fdict * 32 + flevel * 64;
    let fcheck = 31 - cmfflg % 31;
    cmfflg += fcheck;
    /*Vec<u8>-controlled version of the output buffer, for dynamic array*/
    outv.push((cmfflg >> 8) as u8);
    outv.push((cmfflg & 255) as u8);
    let deflated = deflate(inp, settings)?;
    let adler32_val = adler32(inp);
    outv.extend_from_slice(&deflated);
    lodepng_add32bit_int(outv, adler32_val);
    Ok(())
}

/* compress using the default or custom zlib function */
pub(crate) fn zlib_compress(
    inp: &[u8],
    settings: &CompressSettings,
) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    lodepng_zlib_compress(&mut out, inp, settings)?;
    Ok(out)
}

/*this is a good tradeoff between speed and compression ratio*/
pub(crate) const DEFAULT_WINDOWSIZE: usize = 2048;
