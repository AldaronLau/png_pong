#[macro_use]
extern crate criterion;

fn png(c: &mut criterion::Criterion, file: &str) {
    let data = std::fs::read(file).expect("Failed to open PNG");
    c.bench_function(file, |b| {
        b.iter(|| {
            let data = std::io::Cursor::new(data.as_slice());
            let decoder = png::Decoder::new(data);
            let (info, mut reader) = decoder.read_info().unwrap();
            let mut buf = vec![0; info.buffer_size()];
            reader.next_frame(&mut buf).unwrap();
            let _ = buf;
        })
    });
}

fn png_decode(c: &mut criterion::Criterion) {
    for f in comparison::FILE_PATHS {
        png(c, f)
    }
}

criterion_group!(benches, png_decode);
criterion_main!(benches);
