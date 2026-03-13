use image::linear_image::LinearImage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Cpu,
    // Gpu (later)
}

pub trait Node: Send + Sync {
    fn backend(&self) -> Backend;

    fn process_cpu(&self, input: &LinearImage) -> LinearImage;

    // fn process_gpu (later)
}
