#[macro_use]
extern crate criterion;

fn libpng(c: &mut criterion::Criterion, file: &str, alpha: bool) {
    let data =
        std::fs::read(file).expect("Failed to open PNG");
    let data = std::io::Cursor::new(data);
    let decoder = png_pong::StepDecoder::new(data);
    let step = decoder
        .last()
        .expect("No frames in PNG")
        .expect("PNG parsing error");
    if alpha {
        let raster = match step.raster {
            png_pong::PngRaster::Rgba8(ok) => ok,
            _ => unreachable!(),
        };
        c.bench_function(file, |b| {
            b.iter(|| unsafe {
                // 1. Declare png_image struct, 2. Set members to describe image
                let mut png_image = libpng_sys::ffi::png_image {
                    opaque: std::ptr::null_mut(),
                    version: 0,
                    width: raster.width(),
                    height: raster.height(),
                    format: libpng_sys::ffi::PNG_FORMAT_RGBA as u32,
                    flags: 0,
                    colormap_entries: 0,
                    warning_or_error: 0,
                    message: [0; 64],
                };
                // 3. Call png_image_write...
                let mut memory: *mut std::ffi::c_void = std::ptr::null_mut();
                let mut memory_bytes = 0;
                let _r = libpng_sys::ffi::png_image_write_to_memory(&mut png_image, memory, &mut memory_bytes, 0, raster.as_u8_slice().as_ptr().cast(), raster.width() as i32, std::ptr::null());
            })
        });
    } else {
        let raster = match step.raster {
            png_pong::PngRaster::Rgb8(ok) => ok,
            _ => unreachable!(),
        };
        c.bench_function(file, |b| {
            b.iter(|| unsafe {
                // 1. Declare png_image struct, 2. Set members to describe image
                let mut png_image = libpng_sys::ffi::png_image {
                    opaque: std::ptr::null_mut(),
                    version: 0,
                    width: raster.width(),
                    height: raster.height(),
                    format: libpng_sys::ffi::PNG_FORMAT_RGB as u32,
                    flags: 0,
                    colormap_entries: 0,
                    warning_or_error: 0,
                    message: [0; 64],
                };
                // 3. Call png_image_write...
                let mut memory = Vec::with_capacity(100_000_000);
                let mut memory_bytes = 100_000_000;
                let _r = libpng_sys::ffi::png_image_write_to_memory(&mut png_image, memory.as_mut_ptr(), &mut memory_bytes, 0, raster.as_u8_slice().as_ptr().cast(), raster.width() as i32, std::ptr::null());
                memory.set_len(memory_bytes);
            })
        });
    }
}

fn png_pong_encode(c: &mut criterion::Criterion) {
    for (i, f) in comparison::FILE_PATHS.iter().enumerate() {
        libpng(c, f, i % 2 != 0)
    }
}

criterion_group!(benches, png_pong_encode);
criterion_main!(benches);
