#[macro_use]
extern crate criterion;

fn png_pong(c: &mut criterion::Criterion, file: &str) {
    let data = std::fs::read(file).expect("Failed to open PNG");
    let data = std::io::Cursor::new(data);
    let decoder = png_pong::Decoder::new(data).expect("Not PNG").into_steps();
    let step = decoder
        .last()
        .expect("No frames in PNG")
        .expect("PNG parsing error");
    c.bench_function(&format!("PNG Pong Encoder: {}", file), |b| {
        b.iter(|| {
            let mut out_data = Vec::new();
            let mut encoder =
                png_pong::Encoder::new(&mut out_data).into_step_enc();
            encoder.encode(&step).expect("Failed to add frame");
        })
    });
}

fn png_pong_encode(c: &mut criterion::Criterion) {
    for f in comparison::FILE_PATHS {
        png_pong(c, f)
    }
}

criterion_group!(benches, png_pong_encode);
criterion_main!(benches);
