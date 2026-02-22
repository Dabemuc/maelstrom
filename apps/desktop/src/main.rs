use graph::Graph;
use ops::exposure::Exposure;
use image::LinearImage;

fn main() {
    let img = LinearImage::new(100, 100);

    let mut graph = Graph::new();
    graph.add_node(Exposure { ev: 1.0 });

    let result = graph.execute(img, graph::Backend::Cpu);

    println!("Processed image: {}x{}", result.width, result.height);
}
