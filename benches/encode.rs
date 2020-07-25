#[macro_use]
extern crate criterion;

fn encode(c: &mut criterion::Criterion) {
    let data = std::fs::read("./tests/png/4.png").expect("Failed to open PNG");
    let data = std::io::Cursor::new(data);
    let decoder = png_pong::decode::StepDecoder::new(data).expect("Not PNG");
    let step = decoder
        .last()
        .expect("No frames in PNG")
        .expect("PNG parsing error");
    c.bench_function("encoder", |b| {
        b.iter(|| {
            let mut out_data = Vec::new();
            let mut encoder =
                png_pong::encode::StepEncoder::new(&mut out_data, None, 6);
            encoder.encode(&step).expect("Failed to add frame");
        })
    });
}

criterion_group!(benches, encode);
criterion_main!(benches);
