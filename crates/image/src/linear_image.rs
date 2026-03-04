use maelstrom_core::color::{color_space::ColorSpace, working_space::convert_from_workspace};

#[derive(Debug, Clone)]
pub enum WorkingSpace {
    LinearSRgb,
}

#[derive(Debug, Clone)]
pub struct LinearImage {
    pub width: u32,
    pub height: u32,
    pub stride: usize,  // elements per row in f32 (for later gpu processing)
    pub data: Vec<f32>, // RGBARGBARGBA...
    pub space: WorkingSpace,
}

impl LinearImage {
    pub fn new(width: u32, height: u32, space: WorkingSpace) -> Self {
        let stride = (width * 4) as usize; // No padding/alignment for now. Row is exactly 4 channels wide.

        Self {
            width,
            height,
            stride,
            data: vec![0.0; (width * height * 4) as usize],
            space,
        }
    }

    pub fn to_pixels(&self) -> Vec<u8> {
        let pixel_count = self.width as usize * self.height as usize;
        let mut pixels = Vec::with_capacity(pixel_count * 4);

        for chunk in self.data.chunks_exact(4) {
            let r = convert_from_workspace(ColorSpace::Srgb, chunk[0]);
            let g = convert_from_workspace(ColorSpace::Srgb, chunk[1]);
            let b = convert_from_workspace(ColorSpace::Srgb, chunk[2]);
            let a = (chunk[3].clamp(0.0, 1.0) * 255.0) as u8;

            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
            pixels.push(a);
        }

        pixels
    }
}
