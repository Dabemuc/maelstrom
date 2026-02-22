use crate::color::color_space::ColorSpace;

/// Converts any supported ColorSpace into linear sRGB working space.
pub fn convert_to_workspace(space: ColorSpace, v: u8) -> f32 {
    match space {
        ColorSpace::Srgb => srgb_to_linear(v),
    }
}

/// Converts from linear sRGB working space to any supported ColorSpace.
pub fn convert_from_workspace(space: ColorSpace, v: f32) -> u8 {
    match space {
        ColorSpace::Srgb => linear_to_srgb(v),
    }
}

fn srgb_to_linear(v: u8) -> f32 {
    let c = v as f32 / 255.0;

    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);

    let srgb = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };

    (srgb * 255.0).round() as u8
}
