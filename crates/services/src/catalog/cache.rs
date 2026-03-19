use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use io::catalog::ImageDO;
use maelstrom_core::hash::hash_file;

pub fn compare_cache_to_fs(
    paths_of_images_in_folder: Vec<PathBuf>,
    image_dos_in_catalog: Vec<ImageDO>,
) -> (Vec<PathBuf>, Vec<ImageDO>) {
    let mut fs_hash_map: HashMap<String, PathBuf> = HashMap::new();

    for path in &paths_of_images_in_folder {
        if let Ok(hash) = hash_file(path) {
            fs_hash_map.insert(hash, path.clone());
        }
    }

    let catalog_hash_set: HashSet<String> = image_dos_in_catalog
        .iter()
        .map(|img| img.hash.clone())
        .collect();

    let images_not_in_catalog: Vec<PathBuf> = fs_hash_map
        .iter()
        .filter(|(hash, _)| !catalog_hash_set.contains(*hash))
        .map(|(_, path)| path.clone())
        .collect();

    let fs_hash_set: HashSet<&String> = fs_hash_map.keys().collect();

    let catalog_not_in_fs: Vec<ImageDO> = image_dos_in_catalog
        .into_iter()
        .filter(|img| !fs_hash_set.contains(&img.hash))
        .collect();

    (images_not_in_catalog, catalog_not_in_fs)
}
