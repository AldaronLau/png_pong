include!("../list.rs");

#[macro_use]
extern crate criterion;

fn decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("decode");
    group.sample_size(10);

    for file in FILE_PATHS.iter().copied() {
        let data = std::fs::read(file).expect("Failed to open PNG");

        group.bench_function(file, |b| {
            b.iter(|| {
                let data = std::io::Cursor::new(data.as_slice());
                let decoder =
                    png_pong::Decoder::new(data).expect("Not PNG").into_steps();
                let png_pong::Step { raster, delay: _ } = decoder
                    .last()
                    .expect("No frames in PNG")
                    .expect("PNG parsing error");
                let _ = raster;
            })
        });
    }
}

criterion_group!(benches, decode);
criterion_main!(benches);
