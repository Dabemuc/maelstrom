use graph::node::{Backend, Node};
use image::linear_image::LinearImage;

pub struct DownsampleFixed {
    pub max_width: u32,
    pub max_height: u32,
}

impl Node for DownsampleFixed {
    fn backend(&self) -> Backend {
        Backend::Cpu
    }

    fn process_cpu(&self, input: &LinearImage) -> LinearImage {
        assert!(self.max_width > 0 && self.max_height > 0);

        let width_ratio = self.max_width as f32 / input.width as f32;
        let height_ratio = self.max_height as f32 / input.height as f32;

        // Use the smaller ratio to ensure the image fits inside the border
        let scale = width_ratio.min(height_ratio).min(1.0);

        let new_width = ((input.width as f32) * scale).round().max(1.0) as u32;
        let new_height = ((input.height as f32) * scale).round().max(1.0) as u32;

        let mut output = LinearImage::new(new_width, new_height, input.space.clone());

        let x_ratio = input.width as f32 / new_width as f32;
        let y_ratio = input.height as f32 / new_height as f32;

        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x as f32 * x_ratio).floor() as u32;
                let src_y = (y as f32 * y_ratio).floor() as u32;

                let src_index = (src_y as usize * input.stride) + (src_x as usize * 4);

                let dst_index = (y as usize * output.stride) + (x as usize * 4);

                output.data[dst_index..dst_index + 4]
                    .copy_from_slice(&input.data[src_index..src_index + 4]);
            }
        }

        output
    }
}
