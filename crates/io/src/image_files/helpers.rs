use std::fs;
use std::path::PathBuf;

use crate::image_files::supported_image_file_types::SupportedFileTypes;

pub fn collect_images_in_folder(path: PathBuf) -> Vec<PathBuf> {
    fn collect_recursive(path: &PathBuf, images: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();

                if p.is_dir() {
                    collect_recursive(&p, images);
                } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if SupportedFileTypes::is_supported(ext.to_ascii_lowercase().as_str()) {
                        images.push(p);
                    }
                }
            }
        }
    }

    let mut images = Vec::new();
    collect_recursive(&path, &mut images);
    images
}
