#[macro_use]
extern crate criterion;

fn libpng(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>, file: &str, data: &[u8], alpha: bool) {
    group.bench_function(&format!("PNG Decoder: {}", file), |b| {
        b.iter(|| {
            // 1. Declare png_image struct
            let mut png_image = libpng_sys::ffi::png_image {
                opaque: std::ptr::null_mut(),
                version: libpng_sys::ffi::PNG_IMAGE_VERSION as u32,
                width: 0,
                height: 0,
                format: 0,
                flags: 0,
                colormap_entries: 0,
                warning_or_error: 0,
                message: [0; 64],
            };
            // 2. Begin read
            let _r = libpng_sys::ffi::png_image_begin_read_from_memory(
                &mut png_image,
                data.as_ptr().cast(),
                data.len(),
            );
            if alpha {
                // 3. Set required sample format
                png_image.format = libpng_sys::ffi::PNG_FORMAT_RGBA as u32;
                // 4. Allocate buffer for image
                let mut raster = pix::Raster::<pix::rgb::SRgba8>::with_clear(
                    png_image.width,
                    png_image.height,
                );
                // 5. Call png_image_finish_read
                let row_stride = png_image.width as i32;
                let bg = libpng_sys::ffi::png_color {
                    red: 0,
                    green: 0,
                    blue: 0,
                };
                let _r = libpng_sys::ffi::png_image_finish_read(
                    &mut png_image,
                    &bg,
                    raster.as_u8_slice_mut().as_mut_ptr().cast(),
                    row_stride,
                    std::ptr::null_mut(),
                );
                let _ = raster;
            } else {
                // 3. Set required sample format
                png_image.format = libpng_sys::ffi::PNG_FORMAT_RGB as u32;
                // 4. Allocate buffer for image
                let mut raster = pix::Raster::<pix::rgb::SRgb8>::with_clear(
                    png_image.width,
                    png_image.height,
                );
                // 5. Call png_image_finish_read
                let row_stride = png_image.width as i32;
                let bg = libpng_sys::ffi::png_color {
                    red: 0,
                    green: 0,
                    blue: 0,
                };
                let _r = libpng_sys::ffi::png_image_finish_read(
                    &mut png_image,
                    &bg,
                    raster.as_u8_slice_mut().as_mut_ptr().cast(),
                    row_stride,
                    std::ptr::null_mut(),
                );
                let _ = raster;
            }
        })
    });
}

fn libpng_decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("png");
    group.sample_size(10);
    for file in comparison::FILE_PATHS {
        let data = std::fs::read(file).expect("Failed to open PNG");
        libpng(&mut group, file, &data, i % 2 != 0 /* FIXME: Detect? */);
    }
    group.finish();
}

criterion_group!(benches, libpng_decode);
criterion_main!(benches);
