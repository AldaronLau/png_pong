#[macro_use]
extern crate criterion;

fn imagefmt(c: &mut criterion::Criterion, file: &str, alpha: bool) {
    let data = std::fs::read(file).expect("Failed to open PNG");
    let data = std::io::Cursor::new(data);
    let decoder = png_pong::Decoder::new(data).expect("Not PNG").into_steps();
    let step = decoder
        .last()
        .expect("No frames in PNG")
        .expect("PNG parsing error");
    if alpha {
        let raster = match step.raster {
            png_pong::PngRaster::Rgba8(ok) => ok,
            _ => unreachable!(),
        };
        c.bench_function(file, |b| {
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
    } else {
        let raster = match step.raster {
            png_pong::PngRaster::Rgb8(ok) => ok,
            _ => unreachable!(),
        };
        c.bench_function(file, |b| {
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
    }
}

fn imagefmt_encode(c: &mut criterion::Criterion) {
    for (i, f) in comparison::FILE_PATHS.iter().enumerate() {
        imagefmt(c, f, i % 2 != 0)
    }
}

criterion_group!(benches, imagefmt_encode);
criterion_main!(benches);
