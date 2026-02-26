use graph::node::{Backend, Node};
use image::linear_image::LinearImage;

pub struct Downsample {
    pub scale: f32, // e.g. 0.1
}

impl Node for Downsample {
    fn backend(&self) -> Backend {
        Backend::Cpu
    }

    fn process_cpu(&self, input: &LinearImage) -> LinearImage {
        // Nearest Neghbor
        assert!(self.scale > 0.0);

        let new_width = ((input.width as f32) * self.scale).round().max(1.0) as u32;
        let new_height = ((input.height as f32) * self.scale).round().max(1.0) as u32;

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

// Box Filtering
// fn process_cpu(&self, input: &LinearImage) -> LinearImage {
//     assert!(self.scale > 0.0 && self.scale <= 1.0);

//     let new_width = ((input.width as f32) * self.scale).round().max(1.0) as u32;
//     let new_height = ((input.height as f32) * self.scale).round().max(1.0) as u32;

//     let mut output = LinearImage::new(new_width, new_height, input.space.clone());

//     let scale_x = input.width as f32 / new_width as f32;
//     let scale_y = input.height as f32 / new_height as f32;

//     for y in 0..new_height {
//         for x in 0..new_width {
//             let src_x0 = (x as f32 * scale_x).floor() as u32;
//             let src_x1 = ((x + 1) as f32 * scale_x).ceil() as u32;
//             let src_y0 = (y as f32 * scale_y).floor() as u32;
//             let src_y1 = ((y + 1) as f32 * scale_y).ceil() as u32;

//             let mut accum = [0.0f32; 4];
//             let mut count = 0.0;

//             for sy in src_y0..src_y1.min(input.height) {
//                 for sx in src_x0..src_x1.min(input.width) {
//                     let idx = (sy as usize * input.stride) + (sx as usize * 4);
//                     for c in 0..4 {
//                         accum[c] += input.data[idx + c];
//                     }
//                     count += 1.0;
//                 }
//             }

//             let dst_idx = (y as usize * output.stride) + (x as usize * 4);
//             for c in 0..4 {
//                 output.data[dst_idx + c] = accum[c] / count;
//             }
//         }
//     }

//     output
// }
