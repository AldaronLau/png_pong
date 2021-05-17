use comparison::FILE_PATHS;

#[macro_use]
extern crate criterion;

fn decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("decode_lodepng");
    group.sample_size(10);

    for (i, file) in FILE_PATHS.iter().copied().enumerate() {
        let data = std::fs::read(file).expect("Failed to open PNG");
        let data = data.as_slice();

        if i % 2 == 0 {
            group.bench_function(file, |b| {
                b.iter(|| {
                    let image =
                        lodepng::decode_memory(data, lodepng::ColorType::RGBA, 8)
                            .expect("Failed to decode with lodepng");
                    let _ = image;
                })
            });
        } else {
            group.bench_function(file, |b| {
                b.iter(|| {
                    let image =
                        lodepng::decode_memory(data, lodepng::ColorType::RGB, 8)
                            .expect("Failed to decode with lodepng");
                    let _ = image;
                })
            });
        }
    }
}

fn encode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("encode_lodepng");
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
                    let out_data = lodepng::encode_memory(
                        raster.as_u8_slice(),
                        raster.width() as usize,
                        raster.height() as usize,
                        lodepng::ColorType::RGB,
                        8,
                    )
                    .expect("Failed to encode with lodepng");
                    let _ = out_data;
                })
            });
        } else {
            let raster = match raster {
                png_pong::PngRaster::Rgba8(raster) => raster,
                _ => unreachable!(),
            };
            
            group.bench_function(file, |b| {
                b.iter(|| {
                    let out_data = lodepng::encode_memory(
                        raster.as_u8_slice(),
                        raster.width() as usize,
                        raster.height() as usize,
                        lodepng::ColorType::RGBA,
                        8,
                    )
                    .expect("Failed to encode with lodepng");
                    let _ = out_data;
                })
            });
        };
    }
}

criterion_group!(benches, encode, decode);
criterion_main!(benches);
