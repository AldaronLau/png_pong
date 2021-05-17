use comparison::FILE_PATHS;

#[macro_use]
extern crate criterion;

fn encode_sys(raster: &png_pong::PngRaster, alpha: bool) {
    if alpha {
        let raster = match raster {
            png_pong::PngRaster::Rgba8(ok) => ok,
            _ => unreachable!(),
        };

        // 1. Declare png_image struct, 2. Set members to describe image
        let mut png_image = libpng_sys::ffi::png_image {
            opaque: std::ptr::null_mut(),
            version: libpng_sys::ffi::PNG_IMAGE_VERSION as u32,
            width: raster.width(),
            height: raster.height(),
            format: libpng_sys::ffi::PNG_FORMAT_RGBA as u32,
            flags: 0,
            colormap_entries: 0,
            warning_or_error: 0,
            message: [0; 64],
        };
        // 3. Call png_image_write...
        let memory: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut memory_bytes = 0;
        let _r = unsafe { libpng_sys::ffi::png_image_write_to_memory(
            &mut png_image,
            memory,
            &mut memory_bytes,
            0,
            raster.as_u8_slice().as_ptr().cast(),
            0, // raster.width() as i32 * 4, // 0 probably has the same effect.
            std::ptr::null(),
        ) };
    } else {
        let raster = match raster {
            png_pong::PngRaster::Rgb8(ok) => ok,
            _ => unreachable!(),
        };

        // 1. Declare png_image struct, 2. Set members to describe image
        let mut png_image = libpng_sys::ffi::png_image {
            opaque: std::ptr::null_mut(),
            version: libpng_sys::ffi::PNG_IMAGE_VERSION as u32,
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
        let _r = unsafe { libpng_sys::ffi::png_image_write_to_memory(
            &mut png_image,
            memory.as_mut_ptr(),
            &mut memory_bytes,
            0,
            raster.as_u8_slice().as_ptr().cast(),
            0, // raster.width() as i32 * 3, // 0 probably has the same effect.
            std::ptr::null(),
        ) };
    }
}

fn decode_sys(data: &[u8], alpha: bool) {
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
    let _r = unsafe {libpng_sys::ffi::png_image_begin_read_from_memory(
        &mut png_image,
        data.as_ptr().cast(),
        data.len(),
    ) };
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
        let _r = unsafe { libpng_sys::ffi::png_image_finish_read(
            &mut png_image,
            &bg,
            raster.as_u8_slice_mut().as_mut_ptr().cast(),
            row_stride,
            std::ptr::null_mut(),
        ) };
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
        let _r = unsafe { libpng_sys::ffi::png_image_finish_read(
            &mut png_image,
            &bg,
            raster.as_u8_slice_mut().as_mut_ptr().cast(),
            row_stride,
            std::ptr::null_mut(),
        ) };
        let _ = raster;
    }
}

fn decode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("decode_libpng");
    group.sample_size(10);

    for (i, file) in FILE_PATHS.iter().copied().enumerate() {
        let data = std::fs::read(file).expect("Failed to open PNG");
        let data = data.as_slice();

        if i % 2 == 0 {
            group.bench_function(file, |b| {
                b.iter(|| {
                    decode_sys(data, false /* rgb */ )
                })
            });
        } else {
            group.bench_function(file, |b| {
                b.iter(|| {
                    decode_sys(data, true /* rbga */ )
                })
            });
        }
    }
}

fn encode(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("encode_libpng");
    group.sample_size(10);

    for (i, file) in FILE_PATHS.iter().copied().enumerate() {
        let data = std::fs::read(file).expect("Failed to open PNG");
        let data = std::io::Cursor::new(data);
        let decoder =
            png_pong::Decoder::new(data).expect("Not PNG").into_steps();
        let png_pong::Step { raster, .. } = decoder
            .last()
            .expect("No frames in PNG")
            .expect("PNG parsing error");
        if i % 2 == 0 {
            group.bench_function(file, |b| {
                b.iter(|| {
                    encode_sys(&raster, false)
                })
            });
        } else {
            group.bench_function(file, |b| {
                b.iter(|| {
                    encode_sys(&raster, true)
                })
            });
        };
    }
}

criterion_group!(benches, encode, decode);
criterion_main!(benches);
