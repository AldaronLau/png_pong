#[macro_use]
extern crate criterion;

fn png_pong(c: &mut criterion::Criterion, file: &str) {
    let data = std::fs::read(file).expect("Failed to open PNG");
    c.bench_function(file, |b| {
        b.iter(|| {
            let data = std::io::Cursor::new(data.as_slice());
            let decoder = png_pong::StepDecoder::new(data);
            let png_pong::Step { raster, delay: _ } = decoder
                .last()
                .expect("No frames in PNG")
                .expect("PNG parsing error");
            let _ = raster;
        })
    });
}

fn png_pong_decode(c: &mut criterion::Criterion) {
    for f in comparison::FILE_PATHS {
        png_pong(c, f)
    }
}

criterion_group!(benches, png_pong_decode);
criterion_main!(benches);
