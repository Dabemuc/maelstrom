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
}
