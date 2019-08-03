// Inflator (Decompressor)

use super::*;
use inflate::inflate_bytes;

pub(crate) fn lodepng_inflatev(
    inp: &[u8],
    _settings: &DecompressSettings,
) -> Result<Vec<u8>, Error> {
    match inflate_bytes(inp) {
        Ok(rtn) => Ok(rtn),
        Err(e) => {
            eprintln!("Inflate Failure: {}", e);
            Err(Error(52))
        }
    }
}

pub(super) fn inflate(
    inp: &[u8],
    settings: &DecompressSettings,
) -> Result<Vec<u8>, Error> {
    lodepng_inflatev(inp, settings)
}
