use graph::node::Backend;
use graph::graph::Graph;
use io::png::{load_png, save_png};
use ops::exposure::Exposure;

fn main() {
    let img = load_png(concat!(env!("CARGO_MANIFEST_DIR"), "/test.png")).unwrap();

    let mut graph = Graph::new();
    graph.add_node(Exposure { ev: 2.0 });

    let result = graph.execute(img, Backend::Cpu);

    println!("Processed image: {}x{}", result.width, result.height);
    save_png(&result, concat!(env!("CARGO_MANIFEST_DIR"), "/output.png")).unwrap();
}
