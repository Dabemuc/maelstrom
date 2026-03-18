use std::path::PathBuf;

use io::catalog::catalog::Catalog;
use io::image_files::helpers::scan_folder_images;
use previews::preview_generation;

use crate::cache::compare_cache_to_fs;
use crate::error::ServiceError;
use crate::preview_data::preview_data_from_image_do;
use crate::types::SelectionSyncResult;

pub async fn sync_selection(
    catalog: &Catalog,
    request_id: u64,
    selected_path: PathBuf,
) -> Result<SelectionSyncResult, ServiceError> {
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
        compare_cache_to_fs(selected_scan.all_image_paths, image_dos.clone());

    let mut generated = Vec::new();
    for path in &images_to_add_to_catalog {
        generated.push(
            preview_generation::generate_preview_for_image(path.clone(), catalog, false).await,
        );
    }

    Ok(SelectionSyncResult {
        request_id,
        selected_path,
        image_dos,
        preview_data,
        images_to_add_to_catalog,
        catalog_image_dos_to_delete,
        generated,
    })
}
