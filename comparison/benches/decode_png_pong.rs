#[macro_use]
extern crate criterion;

fn png_pong(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>, file: &str, data: &[u8]) {
    group.bench_function(&format!("PNG Pong Decoder: {}", file), |b| {
        b.iter(|| {
            let data = std::io::Cursor::new(data);
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

fn png_pong_decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("png_pong");
    group.sample_size(10);
    for file in comparison::FILE_PATHS {
        let data = std::fs::read(file).expect("Failed to open PNG");
        png_pong(&mut group, file, &data);
    }
    group.finish();
}

criterion_group!(benches, png_pong_decode);
criterion_main!(benches);
