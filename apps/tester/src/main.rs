use io::png_loaeder::load_png;
use graph::{Graph, Backend};
use ops::exposure::Exposure;

fn main() {
    let img = load_png(concat!(env!("CARGO_MANIFEST_DIR"), "/test.png")).unwrap();

    let mut graph = Graph::new();
    graph.add_node(Exposure { ev: 1.0 });

    let result = graph.execute(img, Backend::Cpu);

    println!("Processed image: {}x{}", result.width, result.height);
}
