pub fn srgb_to_linear(v: u8) -> f32 {
    let c = v as f32 / 255.0;

    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);

    let srgb = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };

    (srgb * 255.0).round() as u8
}