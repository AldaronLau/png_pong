use std::{fs::File, io::BufWriter};

use pix::{hwb::SHwb8, rgb::SRgb8, Raster};
use png_pong::Encoder;

fn main() {
    let mut r = Raster::with_clear(256, 256);
    for (y, row) in r.rows_mut(()).enumerate() {
        for (x, p) in row.iter_mut().enumerate() {
            let h = ((x + y) >> 1) as u8;
            let w = y.saturating_sub(x) as u8;
            let b = x.saturating_sub(y) as u8;
            *p = SHwb8::new(h, w, b);
        }
    }
    // Convert to SRgb8 pixel format
    let raster = Raster::<SRgb8>::with_raster(&r);

    // Save PNG File Out
    let writer = BufWriter::new(File::create("out.png").unwrap());
    let mut encoder = Encoder::new(writer).into_step_enc();
    encoder.still(&raster).expect("Failed to write PNG");
}
