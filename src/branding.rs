/// Embedded in the executable so branding never depends on the working directory.
pub fn pixels() -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    decode(include_bytes!("../resources/branding/epok.png"))
}

pub fn lockup_pixels() -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    decode(include_bytes!("../resources/branding/epok-lockup.png"))
}

pub fn splash_pixels() -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    decode(include_bytes!("../resources/branding/epok-splash.png"))
}

pub fn controller_pixels() -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    decode(include_bytes!("../resources/editor/psx-controller.png"))
}

fn decode(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info()?;
    let mut pixels = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut pixels)?;
    pixels.truncate(info.buffer_size());
    match info.color_type {
        png::ColorType::Rgba => {}
        png::ColorType::Rgb => {
            pixels = pixels
                .chunks_exact(3)
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect();
        }
        _ => return Err("Epok artwork must be RGB or RGBA PNG".into()),
    }
    Ok((pixels, info.width, info.height))
}
