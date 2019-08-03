extern crate png_pong;

/*#[bench]
fn encode(/*bencher: &mut test::Bencher*/) {
    let mut data = vec![0u8; 640*480*3];
    for (i, px) in data.iter_mut().enumerate() {
        *px = ((i ^ (13 + i * 17) ^ (i * 13) ^ (i/113 * 11)) >> 5) as u8;
    }
//    bencher.iter(|| {
        png_pong::encode_memory(&data, 640, 480, png_pong::ColorType::RGB, 8).unwrap();
//    });
}*/
