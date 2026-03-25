use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use io::{
    catalog::{
        EditGraph, ImageDO,
        catalog::{CATALOG_FILE_NAME, CATALOG_FOLDER_NAME, Catalog},
        catalog_error::CatalogError,
    },
    image_files::helpers::scan_folder_images,
    import::{ImportDecision, create_import_plan, execute_import_plan},
};

use crate::{ImportStrategy, types::ImportCompletedPayload};

use crate::{
    catalog::catalog_sync::sync_catalog_with_fs_for_dir, error::ServiceError,
    types::CatalogSyncResult,
};

#[derive(Debug, Clone)]
pub struct CatalogService {
    catalog: Catalog,
}

impl CatalogService {
    pub async fn init(config_base: PathBuf) -> Result<Self, ServiceError> {
        // Load catalog file
        if !config_base.exists() {
            println!("User config dir doesnt exists at {:?}", config_base);
            return Err(ServiceError::Catalog(
                CatalogError::MissingCatalogBaseDirectory(config_base.to_str().unwrap().to_owned()),
            ));
        }
        let catalog_root = config_base.join(CATALOG_FOLDER_NAME);

        let catalog = if catalog_root.join(CATALOG_FILE_NAME).exists() {
            println!("Catalog file exists; loading catalog");
            Catalog::load(catalog_root.clone()).await?
        } else {
            println!("default catalog not found, creating at: {:?}", catalog_root);
            Catalog::create(config_base.clone()).await?
        };

        Ok(Self { catalog })
    }

    // TODO: Should eventually be removed
    pub fn get_catalog_ref(&self) -> &Catalog {
        &self.catalog
    }

    pub async fn get_managed_directories(&self) -> Result<Vec<PathBuf>, ServiceError> {
        Ok(self.catalog.get_managed_directories().await?)
    }

    pub async fn add_managed_directory(&self, path: impl AsRef<Path>) -> Result<(), ServiceError> {
        Ok(self.catalog.add_managed_directory(path).await?)
    }

    pub async fn get_edit_graph(&self, content_hash: &str) -> Result<EditGraph, ServiceError> {
        Ok(self.catalog.get_edit_graph(content_hash).await?)
    }

    pub async fn get_all_image_dos_for_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<ImageDO>, ServiceError> {
        Ok(self.catalog.get_all_image_dos_for_path(path).await?)
    }

    pub async fn set_edit_graph(
        &self,
        content_hash: &str,
        graph: &EditGraph,
    ) -> Result<(), ServiceError> {
        Ok(self.catalog.set_edit_graph(content_hash, graph).await?)
    }

    // TODO: Should eventually be removed
    pub fn root(&self) -> &Path {
        self.catalog.root()
    }

    // TODO: Should eventually be removed
    pub fn cache_dir(&self) -> &Path {
        self.catalog.cache_dir()
    }

    // TODO: Should eventually be removed
    pub fn preview_cache_dir(&self) -> &Path {
        self.catalog.preview_cache_dir()
    }

    // TODO: Should eventually be removed
    pub fn develop_cache_dir(&self) -> &Path {
        self.catalog.develop_cache_dir()
    }

    // TODO: Should be renamed (whole process, aka all involved files/methods)
    /// Compares catalog to fs for path and adds new images to catalog
    pub async fn sync_catalog_with_fs_for_dir(
        &self,
        request_id: u64,
        selected_path: PathBuf,
    ) -> Result<CatalogSyncResult, ServiceError> {
        sync_catalog_with_fs_for_dir(&self.catalog, request_id, selected_path).await
    }

    /// Imports fotos into managed directory by copying them according to an import strategy and
    /// adding them to catalog afterwards.
    ///
    /// # Arguments
    ///
    /// * `import_strategy` - The copy strategy of the import
    /// * `import_path` - The path of the directory to import from
    /// * `managed_dir_path` - The path of the managed directory to import into
    pub async fn import_fotos_into_managed_dir_with_strategy(
        &self,
        import_strategy: ImportStrategy,
        import_path: PathBuf,
        managed_dir_path: PathBuf,
    ) -> ImportCompletedPayload {
        // Read all images to import
        let scan_result = scan_folder_images(import_path.clone());
        println!(
            "Importing {} images into {:?}",
            scan_result.all_image_paths.len(),
            import_path.to_str()
        );

        let already_imported_images = match self
            .catalog
            .get_all_image_dos_for_path(managed_dir_path.clone())
            .await
        {
            Ok(value) => value,
            Err(err) => {
                return ImportCompletedPayload {
                    summary: format!("Failed to load already imported images for path: {}", err),
                    imported_items: Vec::new(),
                    root: managed_dir_path.clone(),
                };
            }
        };
        let existing_hashes: HashSet<String> = already_imported_images
            .into_iter()
            .map(|img| img.hash)
            .collect();
        let plan = create_import_plan(
            managed_dir_path.clone(),
            &scan_result,
            &existing_hashes,
            import_strategy,
        );
        let imported_items = plan
            .items
            .iter()
            .filter(|item| item.decision == ImportDecision::Import)
            .cloned()
            .collect();
        let report = execute_import_plan(plan, &self.catalog).await;

        let summary = format!(
            "Imported {} new images out of {} total. Skipped {}, errors {}.",
            report.imported_count,
            scan_result.all_image_paths.len(),
            report.skipped_count,
            report.errors.len()
        );

        ImportCompletedPayload {
            summary,
            imported_items,
            root: managed_dir_path.clone(),
        }
    }
}
