use comparison::FILE_PATHS;

#[macro_use]
extern crate criterion;

fn decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("decode_imagefmt");
    group.sample_size(10);

    for file in FILE_PATHS.iter().copied() {
        let data = std::fs::read(file).expect("Failed to open PNG");
        let data = data.as_slice();

        group.bench_function(file, |b| {
            b.iter(|| {
                let mut data = std::io::Cursor::new(data);
                let image = imagefmt::read_from(&mut data, imagefmt::ColFmt::Auto)
                    .expect("Failed to decode");
                let _ = image;
            })
        });
    }
}

fn encode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("encode_imagefmt");
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
                    // There's no writer API, so this'll have to do.
                    imagefmt::write(
                        "/tmp/imagefmt.png",
                        raster.width() as usize,
                        raster.height() as usize,
                        imagefmt::ColFmt::RGB,
                        raster.as_u8_slice(),
                        imagefmt::ColType::Auto,
                    )
                    .unwrap();
                })
            });
        } else {
            let raster = match raster {
                png_pong::PngRaster::Rgba8(raster) => raster,
                _ => unreachable!(),
            };
            
            group.bench_function(file, |b| {
                b.iter(|| {
                    // There's no writer API, so this'll have to do.
                    imagefmt::write(
                        "/tmp/imagefmt.png",
                        raster.width() as usize,
                        raster.height() as usize,
                        imagefmt::ColFmt::RGBA,
                        raster.as_u8_slice(),
                        imagefmt::ColType::Auto,
                    )
                    .unwrap();
                })
            });
        };
    }
}

criterion_group!(benches, encode, decode);
criterion_main!(benches);
