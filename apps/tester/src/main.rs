use graph::graph::Graph;
use graph::node::Backend;
use io::image_files::png::{load_png, save_png};
use maelstrom_core::color::color_space::ColorSpace;
use ops::exposure::Exposure;

fn main() {
    let img = load_png(concat!(env!("CARGO_MANIFEST_DIR"), "/test.png"), ColorSpace::Srgb).unwrap();

    let mut graph = Graph::new();
    graph.add_node(Exposure { ev: 3.0 });

    let result = graph.execute(img, Backend::Cpu);

    println!("Processed image: {}x{}", result.width, result.height);
    save_png(&result, concat!(env!("CARGO_MANIFEST_DIR"), "/output.png"), ColorSpace::Srgb).unwrap();
}
