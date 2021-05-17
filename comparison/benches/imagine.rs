use comparison::FILE_PATHS;

#[macro_use]
extern crate criterion;

fn decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("decode_imagine");
    group.sample_size(10);

    for file in FILE_PATHS.iter().copied() {
        let data = std::fs::read(file).expect("Failed to open PNG");
        let data = data.as_slice();

        group.bench_function(file, |b| {
            b.iter(|| {
                let image = imagine::png::parse_png_rgba8(&data)
                    .expect("Failed to decode with imagine");
                let _ = image;
            })
        });
    }
}

criterion_group!(benches, decode);
criterion_main!(benches);
