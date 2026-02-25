use graph::graph::Graph;
use graph::node::Backend;
use io::image_files::supported_image_file_types::SupportedFileTypes::PNG;
use maelstrom_core::color::color_space::ColorSpace;
use ops::exposure::Exposure;

fn main() {
    let img = PNG
        .load(
            concat!(env!("CARGO_MANIFEST_DIR"), "/test.png"),
            ColorSpace::Srgb,
        )
        .unwrap();

    let mut graph = Graph::new();
    graph.add_node(Exposure { ev: 3.0 });

    let result = graph.execute(img, Backend::Cpu);

    println!("Processed image: {}x{}", result.width, result.height);
    PNG.save(
        &result,
        concat!(env!("CARGO_MANIFEST_DIR"), "/output.png"),
        ColorSpace::Srgb,
        None,
    )
    .unwrap();
}
