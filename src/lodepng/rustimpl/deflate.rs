//! Deflator (Compressor)                                                  / */
use ::deflate::{deflate_bytes_conf, Compression};

use super::*;

fn deflate_no_compression(data: &[u8]) -> Result<Vec<u8>, Error> {
    /*non compressed deflate block data: 1 bit BFINAL,2 bits BTYPE,(5 bits): it jumps to start of next byte,
    2 bytes LEN, 2 bytes nlen, LEN bytes literal DATA*/
    let numdeflateblocks = (data.len() + 65534) / 65535;
    let mut datapos = 0;
    let mut out = Vec::new();
    for i in 0..numdeflateblocks {
        let bfinal = (i == numdeflateblocks - 1) as usize;
        let btype = 0;
        let firstbyte =
            (bfinal + ((btype & 1) << 1) + ((btype & 2) << 1)) as u8;
        out.push(firstbyte);
        let len = (data.len() - datapos).min(65535);
        let nlen = 65535 - len;
        out.push((len & 255) as u8);
        out.push((len >> 8) as u8);
        out.push((nlen & 255) as u8);
        out.push((nlen >> 8) as u8);
        let mut j = 0;
        while j < 65535 && datapos < data.len() {
            out.push(data[datapos]);
            datapos += 1;
            j += 1
        }
    }
    Ok(out)
}

pub(crate) fn lodepng_deflatev(
    inp: &[u8],
    settings: &CompressSettings,
) -> Result<Vec<u8>, Error> {
    if settings.btype > 2 {
        Err(Error(61))
    } else if settings.btype == 0 {
        deflate_no_compression(inp)
    } else {
        Ok(deflate_bytes_conf(inp, Compression::Default))
    }
}

pub(super) fn deflate(
    inp: &[u8],
    settings: &CompressSettings,
) -> Result<Vec<u8>, Error> {
    lodepng_deflatev(inp, settings)
}
