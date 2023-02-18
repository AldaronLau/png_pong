use std::io::Cursor;

use pix::{
    chan::Ch8,
    el::Pixel,
    gray::SGray8,
    rgb::{SRgb8, SRgba8},
    Raster,
};
use png_pong::{Decoder, Encoder, PngRaster};

fn roundtrip_core<F: Pixel<Chan = Ch8>>(raster_a: PngRaster) -> Raster<F> {
    // Encode as SRgba8
    let mut file = Vec::<u8>::new();
    let mut encoder = Encoder::new(&mut file).into_step_enc();
    encoder.still(&raster_a).unwrap();

    // Decode as SRgba8
    let mut decoder = Decoder::new(Cursor::new(file))
        .expect("Not PNG")
        .into_steps();
    let raster_b: Raster<F> = decoder.next().unwrap().unwrap().raster.into();

    //
    let raster_a: Raster<F> = raster_a.into();

    assert_eq!(raster_a.as_u8_slice().len(), raster_b.as_u8_slice().len());
    assert_eq!(raster_a.as_u8_slice(), raster_b.as_u8_slice());

    raster_b
}

fn roundtrip<F: Pixel<Chan = Ch8>>(filename: &str) -> Raster<F> {
    // Decode as SRgba8
    let file = std::fs::read(filename).unwrap();
    let mut decoder = Decoder::new(Cursor::new(file))
        .expect("Not PNG")
        .into_steps();
    let raster_a = decoder.next().unwrap().unwrap().raster;

    roundtrip_core(raster_a)
}

#[test]
fn crushed() {
    let a = roundtrip::<SRgb8>("tests/png/0.png");
    let b = roundtrip::<SRgb8>("tests/png/1.png");
    let c = roundtrip::<SRgb8>("tests/png/2.png");
    let d = roundtrip::<SRgb8>("tests/png/3.png");
    let aa = roundtrip::<SRgb8>("tests/png/4.png");
    let bb = roundtrip::<SRgb8>("tests/png/5.png");
    let cc = roundtrip::<SRgb8>("tests/png/6.png");
    let dd = roundtrip::<SRgb8>("tests/png/7.png");
    assert_eq!(a.as_u8_slice(), aa.as_u8_slice());
    assert_eq!(b.as_u8_slice(), bb.as_u8_slice());
    assert_eq!(c.as_u8_slice(), cc.as_u8_slice());
    assert_eq!(d.as_u8_slice(), dd.as_u8_slice());
}

#[test]
fn fry() {
    roundtrip::<SRgba8>("tests/png/fry.png");
}

#[test]
fn gray() {
    roundtrip::<SGray8>("tests/png/gray.png");
}

#[test]
fn random() {
    let mut data = vec![0u8; 639 * 479 * 3];
    for (i, px) in data.iter_mut().enumerate() {
        *px = ((i ^ (13 + i * 17) ^ (i * 13) ^ (i / 113 * 11)) >> 5) as u8;
    }

    let raster = PngRaster::Rgb8(Raster::<SRgb8>::with_u8_buffer(
        639,
        479,
        data.as_slice(),
    ));
    roundtrip_core::<SRgb8>(raster);
}

// FIXME: Text
/*
#[test]
fn text_chunks() {
    let mut s = State::new();
    s.encoder.text_compression = 0;
    let longstr = "World 123456789_123456789_123456789_123456789_123456789_123456789_123456789_123456789_123456789_";
    assert!(longstr.len() > 89);
    s.info_png_mut().add_text("Hello", longstr);
    assert_eq!(1, s.info_png().text_keys_cstr().count());

    let raster: pix::Raster<pix::Rgba8> =
        pix::RasterBuilder::new().with_u8_buffer(1, 1, &[0u8, 0, 0, 0][..]);

    let data = s.encode(&raster).unwrap();

    assert!(data.windows(4).any(|w| w == b"tEXt"));

    let mut s = State::new();
    s.read_text_chunks(true);
    s.decode(data).unwrap();
    assert_eq!(1, s.info_png().text_keys_cstr().count());
}*/
