#[macro_use]
extern crate criterion;

fn png(c: &mut criterion::Criterion, file: &str, alpha: bool) {
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
                let mut out_data = Vec::new();
                let mut encoder = png::Encoder::new(
                    &mut out_data,
                    raster.width(),
                    raster.height(),
                );
                encoder.set_color(png::ColorType::RGBA);
                encoder.set_depth(png::BitDepth::Eight);
                let mut writer = encoder.write_header().unwrap();
                writer.write_image_data(&raster.as_u8_slice()).unwrap(); // Save
            })
        });
    } else {
        let raster = match step.raster {
            png_pong::PngRaster::Rgb8(ok) => ok,
            _ => unreachable!(),
        };
        c.bench_function(file, |b| {
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
                writer.write_image_data(&raster.as_u8_slice()).unwrap(); // Save
            })
        });
    }
}

fn png_encode(c: &mut criterion::Criterion) {
    for (i, f) in comparison::FILE_PATHS.iter().enumerate() {
        png(c, f, i % 2 != 0)
    }
}

criterion_group!(benches, png_encode);
criterion_main!(benches);
