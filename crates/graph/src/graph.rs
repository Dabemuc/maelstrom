use crate::node::{Backend, Node};
use image::linear_image::LinearImage;

pub struct Graph {
    nodes: Vec<Box<dyn Node>>,
}

impl Graph {
    pub fn new() -> Self {
        Self { nodes: vec![] }
    }

    pub fn add_node<N: Node + 'static>(&mut self, node: N) {
        self.nodes.push(Box::new(node));
    }

    pub fn execute(&self, mut image: LinearImage, backend: Backend) -> LinearImage {
        for node in &self.nodes {
            match (backend, node.backend()) {
                (Backend::Cpu, Backend::Cpu) => {
                    image = node.process_cpu(&image);
                } //
                  // (Backend::Cpu, _) => {
                  //     panic!("Unsupported backend for now");
                  // }
            }
        }
        image
    }
}
