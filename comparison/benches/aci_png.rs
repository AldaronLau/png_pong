use comparison::FILE_PATHS;

use afi::EncoderV;

#[macro_use]
extern crate criterion;

fn decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("decode_aci");
    group.sample_size(10);

    for (i, file) in FILE_PATHS.iter().copied().enumerate() {
        let data = std::fs::read(file).expect("Failed to open PNG");
        let data = data.as_slice();

        if i % 2 == 0 {
            group.bench_function(file, |b| {
                b.iter(|| {
                    let video = aci_png::decode(&data, afi::ColorChannels::Srgb)
                        .expect("Failed to load PNG");
                    let _ = video;
                })
            });
        } else {
            group.bench_function(file, |b| {
                b.iter(|| {
                    let video = aci_png::decode(&data, afi::ColorChannels::Srgba)
                        .expect("Failed to load PNG");
                    let _ = video;
                })
            });
        }
    }
}

fn encode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("encode_aci");
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
            let _raster = match raster {
                png_pong::PngRaster::Rgb8(raster) => raster,
                _ => unreachable!(),
            };
            /*
            group.bench_function(file, |b| {
                b.iter(|| {
                    let raster = match step.raster {
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
                    });
                })
            });*/
        } else {
            let raster = match raster {
                png_pong::PngRaster::Rgba8(raster) => raster,
                _ => unreachable!(),
            };
            
            group.bench_function(file, |b| {
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
        };
    }
}

criterion_group!(benches, encode, decode);
criterion_main!(benches);
