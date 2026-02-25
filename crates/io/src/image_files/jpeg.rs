use image::{ExtendedColorType, ImageBuffer, ImageReader, Rgb, codecs::jpeg::JpegEncoder};
use maelstrom_core::color::color_space::ColorSpace;
use maelstrom_core::color::working_space::{convert_from_workspace, convert_to_workspace};
use maelstrom_image::linear_image::{LinearImage, WorkingSpace};
use std::fs::File;
use std::io::BufWriter;

pub fn load_jpeg(path: &str, space: ColorSpace) -> Result<LinearImage, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path)?.decode()?;
    let rgb = img.to_rgb8();

    let (width, height) = rgb.dimensions();

    let mut output = LinearImage::new(width, height, WorkingSpace::LinearSRgb);

    for y in 0..height {
        for x in 0..width {
            let pixel = rgb.get_pixel(x, y);

            let r = convert_to_workspace(space, pixel[0]);
            let g = convert_to_workspace(space, pixel[1]);
            let b = convert_to_workspace(space, pixel[2]);
            let a = 1.0; // JPEG has no alpha

            let idx = (y as usize * output.stride) + (x as usize * 4);

            output.data[idx] = r;
            output.data[idx + 1] = g;
            output.data[idx + 2] = b;
            output.data[idx + 3] = a;
        }
    }

    Ok(output)
}

pub fn load_jpeg_colorspace(_path: &str) -> ColorSpace {
    // For now we assume JPEG is always sRGB
    ColorSpace::Srgb
}

pub fn save_jpeg(
    img: &LinearImage,
    path: &str,
    space: ColorSpace,
    quality: u8, // 1–100
) -> Result<(), Box<dyn std::error::Error>> {
    match space {
        ColorSpace::Srgb => save_jpeg_srgb(img, path, space, quality),
    }
}

fn save_jpeg_srgb(
    img: &LinearImage,
    path: &str,
    space: ColorSpace,
    quality: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(img.width, img.height);

    for y in 0..img.height {
        for x in 0..img.width {
            let idx = y as usize * img.stride + x as usize * 4;

            let r = convert_from_workspace(space, img.data[idx]);
            let g = convert_from_workspace(space, img.data[idx + 1]);
            let b = convert_from_workspace(space, img.data[idx + 2]);

            buffer.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = JpegEncoder::new_with_quality(writer, quality);

    encoder.encode(&buffer, img.width, img.height, ExtendedColorType::Rgb8)?;

    Ok(())
}
