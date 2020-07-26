#[macro_use]
extern crate criterion;

fn imagefmt(c: &mut criterion::Criterion, file: &str) {
    let data = std::fs::read(file).expect("Failed to open PNG");
    c.bench_function(file, |b| {
        b.iter(|| {
            let mut data = std::io::Cursor::new(data.as_slice());
            let image = imagefmt::read_from(&mut data, imagefmt::ColFmt::Auto)
                .expect("Failed to decode");
            let _ = image;
        })
    });
}

fn imagefmt_decode(c: &mut criterion::Criterion) {
    for f in comparison::FILE_PATHS {
        imagefmt(c, f)
    }
}

criterion_group!(benches, imagefmt_decode);
criterion_main!(benches);
