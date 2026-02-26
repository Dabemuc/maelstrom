use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use io::catalog::ImageDO;
use maelstrom_core::hash::hash_file;

/// Returns:
/// 1. Paths of images that are in the folder but not in the catalog
/// 2. ImageDOs that are in the catalog but not found in the folder
pub fn compare_cache_to_fs(
    paths_of_images_in_folder: Vec<PathBuf>,
    image_dos_in_catalog: Vec<ImageDO>,
) -> (Vec<PathBuf>, Vec<ImageDO>) {
    // Build hash -> PathBuf map for filesystem images
    let mut fs_hash_map: HashMap<String, PathBuf> = HashMap::new();

    for path in &paths_of_images_in_folder {
        if let Ok(hash) = hash_file(path) {
            fs_hash_map.insert(hash, path.clone());
        }
    }

    // Build hash set for catalog entries
    let catalog_hash_set: HashSet<String> = image_dos_in_catalog
        .iter()
        .map(|img| img.hash.clone())
        .collect();

    // Images in folder but NOT in catalog
    let images_not_in_catalog: Vec<PathBuf> = fs_hash_map
        .iter()
        .filter(|(hash, _)| !catalog_hash_set.contains(*hash))
        .map(|(_, path)| path.clone())
        .collect();

    // Images in catalog but NOT in folder
    let fs_hash_set: HashSet<&String> = fs_hash_map.keys().collect();

    let catalog_not_in_fs: Vec<ImageDO> = image_dos_in_catalog
        .into_iter()
        .filter(|img| !fs_hash_set.contains(&img.hash))
        .collect();

    (images_not_in_catalog, catalog_not_in_fs)
}
