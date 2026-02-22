use crate::common::linear_srgb::{linear_to_srgb, srgb_to_linear};
use image::{ImageBuffer, ImageReader, Rgba};
use maelstrom_image::linear_image::LinearImage;

pub fn load_png(path: &str) -> Result<LinearImage, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path)?.decode()?;
    let rgba = img.to_rgba8();

    let (width, height) = rgba.dimensions();

    let mut output = LinearImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);

            let r = srgb_to_linear(pixel[0]);
            let g = srgb_to_linear(pixel[1]);
            let b = srgb_to_linear(pixel[2]);
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

pub fn save_png(img: &LinearImage, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(img.width, img.height);

    for y in 0..img.height {
        for x in 0..img.width {
            let idx = y as usize * img.stride + x as usize * 4;

            let r = linear_to_srgb(img.data[idx]);
            let g = linear_to_srgb(img.data[idx + 1]);
            let b = linear_to_srgb(img.data[idx + 2]);
            let a = (img.data[idx + 3] * 255.0).round() as u8;

            buffer.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }

    buffer.save(path)?;

    Ok(())
}
