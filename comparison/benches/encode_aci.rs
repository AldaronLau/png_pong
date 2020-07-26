#[macro_use]
extern crate criterion;

use afi::EncoderV;

fn aci(c: &mut criterion::Criterion, file: &str, alpha: bool) {
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
                let mut encoder = aci_png::PngEncoder::new(&afi::Video::new(
                    afi::ColorChannels::Srgba,
                    (raster.width() as u16, raster.height() as u16),
                    1,
                ));
                encoder.run(&afi::VFrame(raster.as_u8_slice().to_vec()));
                let out_data = encoder.end();
                let _ = out_data;
            })
        });
    } else {
        /*let raster = match step.raster {
            png_pong::PngRaster::Rgb8(ok) => ok,
            _ => unreachable!(),
        };
        c.bench_function(file, |b| {
            b.iter(|| {
                let mut encoder = aci_png::PngEncoder::new(&afi::Video::new(afi::ColorChannels::Srgb, (raster.width() as u16, raster.height() as u16), 1));
                encoder.run(&afi::VFrame(raster.as_u8_slice().to_vec()));
                let out_data = encoder.end();
                let _ = out_data;
            })
        });*/
    }
}

fn aci_encode(c: &mut criterion::Criterion) {
    for (i, f) in comparison::FILE_PATHS.iter().enumerate() {
        aci(c, f, i % 2 != 0)
    }
}

criterion_group!(benches, aci_encode);
criterion_main!(benches);
