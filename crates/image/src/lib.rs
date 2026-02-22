#[derive(Clone)]
pub struct LinearImage {
    pub width: u32,
    pub height: u32,
    // pub stride: usize,  // GPU alignment ???
    pub data: Vec<f32>, // RGBRGBRGB...
}

impl LinearImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; (width * height * 3) as usize],
        }
    }
}
