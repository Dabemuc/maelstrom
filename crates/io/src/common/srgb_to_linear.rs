pub fn srgb_to_linear(v: u8) -> f32 {
    let c = v as f32 / 255.0;

    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}