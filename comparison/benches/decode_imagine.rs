#[macro_use]
extern crate criterion;

fn imagine(c: &mut criterion::Criterion, file: &str) {
    let data = std::fs::read(file).expect("Failed to open PNG");
    c.bench_function(file, |b| {
        b.iter(|| {
            let image = imagine::png::parse_png_rgba8(&data)
                .expect("Failed to decode with imagine");
            let _ = image;
        })
    });
}

fn imagine_decode(c: &mut criterion::Criterion) {
    for f in comparison::FILE_PATHS {
        imagine(c, f)
    }
}

criterion_group!(benches, imagine_decode);
criterion_main!(benches);
