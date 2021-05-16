#[macro_use]
extern crate criterion;

fn png(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>, file: &str, data: &[u8]) {
    group.bench_function(&format!("PNG Decoder: {}", file), |b| {
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

fn png_decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("png");
    group.sample_size(10);
    for file in comparison::FILE_PATHS {
        let data = std::fs::read(file).expect("Failed to open PNG");
        png(&mut group, file, &data);
    }
    group.finish();
}

criterion_group!(benches, png_decode);
criterion_main!(benches);
