use comparison::FILE_PATHS;

#[macro_use]
extern crate criterion;

fn decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("decode_png");
    group.sample_size(10);

    for file in FILE_PATHS.iter().copied() {
        let data = std::fs::read(file).expect("Failed to open PNG");
        let data = data.as_slice();

        group.bench_function(file, |b| {
            b.iter(|| {
                let data = std::io::Cursor::new(data);
                let decoder = png::Decoder::new(data);
                let (info, mut reader) = decoder.read_info().unwrap();
                let mut buf = vec![0; info.buffer_size()];
                reader.next_frame(&mut buf).unwrap();
                let _ = buf;
            })
        });
    }
}

fn encode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("encode_png");
    group.sample_size(10);

    for (i, file) in FILE_PATHS.iter().copied().enumerate() {
        let data = std::fs::read(file).expect("Failed to open PNG");
        let data = std::io::Cursor::new(data);
        let decoder =
            png_pong::Decoder::new(data).expect("Not PNG").into_steps();
        let png_pong::Step { raster, .. } = decoder
            .last()
            .expect("No frames in PNG")
            .expect("PNG parsing error");
        if i % 2 == 0 {
            let raster = match raster {
                png_pong::PngRaster::Rgb8(raster) => raster,
                _ => unreachable!(),
            };
            
            group.bench_function(file, |b| {
                b.iter(|| {
                    let mut out_data = Vec::new();
                    let mut encoder = png::Encoder::new(
                        &mut out_data,
                        raster.width(),
                        raster.height(),
                    );
                    encoder.set_color(png::ColorType::RGB);
                    encoder.set_depth(png::BitDepth::Eight);
                    let mut writer = encoder.write_header().unwrap();
                    writer.write_image_data(&raster.as_u8_slice()).unwrap();
                })
            });
        } else {
            let raster = match raster {
                png_pong::PngRaster::Rgba8(raster) => raster,
                _ => unreachable!(),
            };
            
            group.bench_function(file, |b| {
                b.iter(|| {
                    let mut out_data = Vec::new();
                    let mut encoder = png::Encoder::new(
                        &mut out_data,
                        raster.width(),
                        raster.height(),
                    );
                    encoder.set_color(png::ColorType::RGBA);
                    encoder.set_depth(png::BitDepth::Eight);
                    let mut writer = encoder.write_header().unwrap();
                    writer.write_image_data(&raster.as_u8_slice()).unwrap();
                })
            });
        };
    }
}

criterion_group!(benches, encode, decode);
criterion_main!(benches);
