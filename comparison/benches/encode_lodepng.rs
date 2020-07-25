#[macro_use]
extern crate criterion;

fn lodepng(c: &mut criterion::Criterion, file: &str, alpha: bool) {
    let data =
        std::fs::read(file).expect("Failed to open PNG");
    let data = std::io::Cursor::new(data);
    let decoder = png_pong::StepDecoder::new(data);
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
                let mut out_data = lodepng::encode_memory(raster.as_u8_slice(), raster.width() as usize, raster.height() as usize, lodepng::ColorType::RGBA, 8).expect("Failed to encode with lodepng");
                let _ = out_data;
            })
        });
    } else {
        let raster = match step.raster {
            png_pong::PngRaster::Rgb8(ok) => ok,
            _ => unreachable!(),
        };
        c.bench_function(file, |b| {
            b.iter(|| {
                let mut out_data = lodepng::encode_memory(raster.as_u8_slice(), raster.width() as usize, raster.height() as usize, lodepng::ColorType::RGB, 8).expect("Failed to encode with lodepng");
                let _ = out_data;
            })
        });
    }
}

fn lodepng_encode(c: &mut criterion::Criterion) {
    for (i, f) in comparison::FILE_PATHS.iter().enumerate() {
        lodepng(c, f, i % 2 != 0)
    }
}

criterion_group!(benches, lodepng_encode);
criterion_main!(benches);
