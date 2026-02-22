use graph::{Node, Backend};
use image::LinearImage;

pub struct Exposure {
    pub ev: f32,
}

impl Node for Exposure {
    fn backend(&self) -> Backend {
        Backend::Cpu
    }

    fn process_cpu(&self, input: &LinearImage) -> LinearImage {
        let mut output = input.clone();
        let multiplier = 2.0f32.powf(self.ev);

        for pixel in output.data.iter_mut() {
            *pixel *= multiplier;
        }

        output
    }
}