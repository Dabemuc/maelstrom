use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use io::catalog::ImageDO;
use io::catalog::catalog::Catalog;
use io::image_files::helpers::scan_folder_images;
use maelstrom_core::hash::hash_file;
use crate::catalog::preview_data::preview_data_from_image_do;
use crate::error::ServiceError;
use crate::types::CatalogSyncResult;

pub async fn sync_catalog_with_fs_for_dir(
    catalog: &Catalog,
    request_id: u64,
    selected_path: PathBuf,
) -> Result<CatalogSyncResult, ServiceError> {
    let image_dos = catalog
        .get_all_image_dos_for_path(&selected_path)
        .await
        .map_err(ServiceError::Catalog)?;

    let preview_data = image_dos
        .iter()
        .map(|image_do| preview_data_from_image_do(catalog, image_do))
        .collect();

    let selected_scan = scan_folder_images(selected_path.clone());
    let (images_to_add_to_catalog, catalog_image_dos_to_delete) =
        compare_catalog_to_fs(selected_scan.all_image_paths, image_dos.clone());

    Ok(CatalogSyncResult {
        request_id,
        selected_path,
        image_dos,
        preview_data,
        images_to_add_to_catalog,
        catalog_image_dos_to_delete,
        generated: vec![],
    })
}

fn compare_catalog_to_fs(
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
