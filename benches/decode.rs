#[macro_use]
extern crate criterion;

fn decode(c: &mut criterion::Criterion) {
    let data = std::fs::read("./tests/png/4.png").expect("Failed to open PNG");
    c.bench_function("decoder", |b| {
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

criterion_group!(benches, decode);
criterion_main!(benches);
