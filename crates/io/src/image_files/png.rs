use image::{ImageBuffer, ImageReader, Rgba};
use maelstrom_core::color::color_space::ColorSpace;
use maelstrom_core::color::working_space::{convert_from_workspace, convert_to_workspace};
use maelstrom_image::linear_image::{LinearImage, WorkingSpace};

pub fn load_png(path: &str, space: ColorSpace) -> Result<LinearImage, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path)?.decode()?;
    let rgba = img.to_rgba8();

    let (width, height) = rgba.dimensions();

    let mut output = LinearImage::new(width, height, WorkingSpace::LinearSRgb);

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);

            let r = convert_to_workspace(space, pixel[0]);
            let g = convert_to_workspace(space, pixel[1]);
            let b = convert_to_workspace(space, pixel[2]);
            let a = pixel[3] as f32 / 255.0;

            let idx = (y as usize * output.stride) + (x as usize * 4);

            output.data[idx] = r;
            output.data[idx + 1] = g;
            output.data[idx + 2] = b;
            output.data[idx + 3] = a;
        }
    }

    Ok(output)
}

pub fn save_png(
    img: &LinearImage,
    path: &str,
    space: ColorSpace,
) -> Result<(), Box<dyn std::error::Error>> {
    match space {
        ColorSpace::Srgb => save_png_srgb(img, path, space),
    }
}

fn save_png_srgb(
    img: &LinearImage,
    path: &str,
    space: ColorSpace,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(img.width, img.height);

    for y in 0..img.height {
        for x in 0..img.width {
            let idx = y as usize * img.stride + x as usize * 4;

            let r = convert_from_workspace(space, img.data[idx]);
            let g = convert_from_workspace(space, img.data[idx + 1]);
            let b = convert_from_workspace(space, img.data[idx + 2]);
            let a = (img.data[idx + 3] * 255.0).round() as u8;

            buffer.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }

    buffer.save(path)?;

    Ok(())
}
