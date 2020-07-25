#[macro_use]
extern crate criterion;

fn lodepng(c: &mut criterion::Criterion, file: &str, alpha: bool) {
    let data = std::fs::read(file).expect("Failed to open PNG");
    let data = data.as_slice();
    if alpha {
        c.bench_function(file, |b| {
            b.iter(|| {
                let image = lodepng::decode_memory(data, lodepng::ColorType::RGBA, 8).expect("Failed to decode with lodepng");
                let _ = image;
            })
        });
    } else {
        c.bench_function(file, |b| {
            b.iter(|| {
                let image = lodepng::decode_memory(data, lodepng::ColorType::RGB, 8).expect("Failed to decode with lodepng");
                let _ = image;
            })
        });
    }
}

fn lodepng_decode(c: &mut criterion::Criterion) {
    for (i, f) in comparison::FILE_PATHS.iter().enumerate() {
        lodepng(c, f, i % 2 != 0)
    }
}

criterion_group!(benches, lodepng_decode);
criterion_main!(benches);
