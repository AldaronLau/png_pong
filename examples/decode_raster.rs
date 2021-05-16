fn main() {
    let data: &[u8] = &std::fs::read("res/icon.png").expect("Failed to open PNG");
    let decoder = png_pong::Decoder::new(data).expect("Not PNG").into_steps();
    let png_pong::Step { raster: _, .. } = decoder
        .last()
        .expect("No frames in PNG")
        .expect("PNG parsing error");
}
