use std::{collections::HashMap, path::PathBuf};

use iced::Task;
use io::{catalog::ImageDO, image_files::helpers::scan_folder_images};
use maelstrom_core::hash::hash_file;

use crate::{app::App, message::Message};

pub fn handle_import_fotos_into_managed_root(app: &mut App, root: PathBuf) -> Task<Message> {
    let Some(catalog) = app.catalog.clone() else {
        println!("Cannot add managed directory: no catalog loaded");
        return Task::none();
    };

    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        let catalog_clone = catalog.clone();
        Task::perform(
            async move {
                // Read all images to import
                let scan_result = scan_folder_images(path.clone());
                println!(
                    "Importing {} images into {:?}",
                    scan_result.all_image_paths.len(),
                    root.to_str()
                );

                // Check if image is already imported. If not import
                let mut imported_count = 0;
                let already_imported_images = match catalog_clone
                    .get_all_image_dos_for_path(root)
                    .await
                {
                    Ok(value) => value,
                    Err(err) => {
                        return format!("Failed to load already imported images for path: {}", err);
                    }
                };
                let already_imported_images_hashmap: HashMap<_, _> = already_imported_images
                    .clone()
                    .into_iter()
                    .map(|img| {
                        let img_clone = img.clone();
                        (img_clone.hash.clone(), img_clone)
                    })
                    .collect();
                for image_path in scan_result.all_image_paths.clone() {
                    if hash_in_dos(
                        hash_file(&image_path).unwrap_or("".to_owned()),
                        already_imported_images_hashmap.clone(),
                    ) {
                        println!("Image at {:?} already imported", image_path.to_str());
                    } else {
                        println!("Importing image from {:?}", image_path.to_str());
                        imported_count += 1;
                        // TODO: Actually do the importing
                    }
                }

                format!(
                    "Imported {} new images out of {} total.",
                    imported_count,
                    scan_result.all_image_paths.len()
                )
            },
            Message::Notification,
        )
    } else {
        println!("FileDialog canceled");
        Task::none()
    }
}

fn hash_in_dos(hash: String, dos: HashMap<String, ImageDO>) -> bool {
    dos.contains_key(&hash)
}
