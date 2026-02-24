use std::path::PathBuf;

use graph::{graph::Graph, node::Backend};
use io::image_files::supported_image_file_types::SupportedFileTypes;
use maelstrom_core::color::color_space::ColorSpace;
use ops::exposure::Exposure;

pub fn generate_preview_for_image(path_to_img: PathBuf) {
    // 1. get file name
    let filename = path_to_filename(path_to_img.clone());

    // 2. check if image supported
    if !SupportedFileTypes::is_supported(&filename) {
        println!(
            "Preview generation failed. Filetype is not supported: {:?}",
            path_to_img
        );
        return;
    }

    // 3. Get type of image
    let image_file_type = SupportedFileTypes::from_filename(&filename).unwrap();

    // 4. Load image
    let image_linear = image_file_type
        .load(
            path_to_img.to_str().unwrap(),
            image_file_type.load_colorspace(path_to_img.to_str().unwrap()),
        )
        .unwrap();

    // 5. Define preview generation graph
    let mut graph = Graph::new();
    // Temporary exposure node. should be downsample and compress
    graph.add_node(Exposure { ev: 3.0 });

    // 6. Generate preview
    let result = graph.execute(image_linear, Backend::Cpu);
    println!(
        "Generated preview for {:?} with resolution: {}x{}",
        path_to_img, result.width, result.height
    );

    // 7. Save preview
    let preview_file_destination = "";
    image_file_type
        .save(
            &result,
            concat!(env!("CARGO_MANIFEST_DIR"), "/output.png"),
            ColorSpace::Srgb,
        )
        .unwrap_or_else(|e| {
            eprintln!(
                "Error saving preview for {:?} at {}: {}",
                path_to_img, preview_file_destination, e
            );
            return;
        });
    println!(
        "Successfully saved preview for {:?} at {}",
        path_to_img, preview_file_destination
    );
}

fn path_to_filename(path: PathBuf) -> String {
    match path.file_name() {
        Some(os_str) => match os_str.to_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!("Failed to convert file name to UTF-8: {:?}", path);
                String::new()
            }
        },
        None => {
            eprintln!("Path has no file name: {:?}", path);
            String::new()
        }
    }
}
