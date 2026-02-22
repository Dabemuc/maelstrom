use image::ImageReader;
use maelstrom_image::linear_image::LinearImage;
use crate::common::srgb_to_linear::srgb_to_linear;

pub fn load_png(path: &str) -> Result<LinearImage, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path)?.decode()?;
    let rgb = img.to_rgb8();

    let (width, height) = rgb.dimensions();

    let mut output = LinearImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = rgb.get_pixel(x, y);

            let r = srgb_to_linear(pixel[0]);
            let g = srgb_to_linear(pixel[1]);
            let b = srgb_to_linear(pixel[2]);

            let idx = (y as usize * output.stride) + (x as usize * 3);

            output.data[idx] = r;
            output.data[idx + 1] = g;
            output.data[idx + 2] = b;
        }
    }

    Ok(output)
}
