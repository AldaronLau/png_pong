#[macro_use]
extern crate criterion;

fn aci(c: &mut criterion::Criterion, file: &str, alpha: bool) {
    let data = std::fs::read(file).expect("Failed to open PNG");
    if alpha {
        c.bench_function(file, |b| {
            b.iter(|| {
                let video = aci_png::decode(&data, afi::ColorChannels::Srgba)
                    .expect("Failed to load PNG");
                let _ = video;
            })
        });
    } else {
        c.bench_function(file, |b| {
            b.iter(|| {
                let video = aci_png::decode(&data, afi::ColorChannels::Srgb)
                    .expect("Failed to load PNG");
                let _ = video;
            })
        });
    }
}

fn aci_decode(c: &mut criterion::Criterion) {
    for (i, f) in comparison::FILE_PATHS.iter().enumerate() {
        aci(c, f, i % 2 != 0)
    }
}

criterion_group!(benches, aci_decode);
criterion_main!(benches);
