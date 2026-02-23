use std::fs;
use std::path::PathBuf;

use crate::image_files::supported_image_file_types::SupportedFileTypes;

pub fn count_images_in_folder(path: PathBuf) -> u32 {
    fn count_recursive(path: &PathBuf) -> u32 {
        let mut total = 0;

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();

                if p.is_dir() {
                    total += count_recursive(&p);
                } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if SupportedFileTypes::is_supported(ext.to_ascii_lowercase().as_str()) {
                        total += 1;
                    }
                }
            }
        }

        total
    }

    count_recursive(&path)
}
