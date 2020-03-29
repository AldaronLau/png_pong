/*extern crate pix;
extern crate png_pong;
use png_pong::*;

fn encode<T: Copy + pix::Format>(
    pixels: &[T],
    in_type: ColorType,
    out_type: ColorType,
) -> Result<Vec<u8>, Error> {
    let raster: pix::Raster<T> =
        pix::RasterBuilder::new().with_pixels(pixels.len() as u32, 1, pixels);
    let mut state = State::new();
    state.set_auto_convert(true);
    state.info_raw.colortype = in_type;
    state.info_raw.set_bitdepth(8);
    state.info_png.color.colortype = out_type;
    state.info_png.color.set_bitdepth(8);
    state.encode(&raster)
}

#[test]
fn bgr() {
    let png =
        encode(&[pix::Rgb8::new(3, 2, 1)], ColorType::BGR, ColorType::RGB)
            .unwrap();
    let img = decode24(&png).unwrap();
    assert_eq!(img.as_slice()[0], pix::Rgb8::new(1, 2, 3));

    let png = encode(
        &[pix::Rgba8::new(3, 2, 1, 111)],
        ColorType::BGRX,
        ColorType::RGB,
    )
    .unwrap();
    let img = decode32(&png).unwrap();
    assert_eq!(
        img.as_slice()[0],
        pix::Rgba8::new(1, 2, 3, 255)
    );
}

#[test]
fn redecode1() {
    let img1 = decode_file("tests/graytest.png", ColorType::GREY, 8).unwrap();
    let img1 = match img1 {
        Image::Grey(a) => a,
        _ => panic!(),
    };
    let png = encode_memory(&img1, ColorType::GREY, 8).unwrap();
    let img2 = decode_memory(&png, ColorType::GREY, 8).unwrap();
    let img2 = match img2 {
        Image::Grey(a) => a,
        _ => panic!(),
    };
    assert_eq!(img1.as_slice(), img2.as_slice());
}

#[test]
fn redecode2() {
    let img1 = decode24_file("tests/fry-test.png").unwrap();
    let png = encode24(&img1).unwrap();
    let img2 = decode24(&png).unwrap();

    assert_eq!(img1.as_slice(), img2.as_slice());
}

#[test]
fn random() {
    let mut data = vec![0u8; 639 * 479 * 3];
    for (i, px) in data.iter_mut().enumerate() {
        *px = ((i ^ (13 + i * 17) ^ (i * 13) ^ (i / 113 * 11)) >> 5) as u8;
    }

    let raster: pix::Raster<pix::Rgb8> =
        pix::RasterBuilder::new().with_u8_buffer(639, 479, data.as_slice());
    let png = encode24(&raster).unwrap();
    let img2 = decode24(&png).unwrap();

    let raster: pix::Raster<pix::Rgb8> =
        pix::RasterBuilder::new().with_u8_buffer(639, 479, data);
    assert_eq!(raster.as_slice(), img2.as_slice());
}

#[test]
fn bgra() {
    let png = encode(
        &[pix::Rgba8::with_alpha(3, 2, 1, 4)]
        ColorType::BGRA,
        ColorType::RGBA,
    )
    .unwrap();
    let img = decode32(&png).unwrap();
    assert_eq!(
        img.as_slice()[0],
        pix::Rgba8::with_alpha(1, 2, 3, 4)
    ;
}

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
