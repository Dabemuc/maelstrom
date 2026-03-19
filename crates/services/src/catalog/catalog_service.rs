use std::path::{Path, PathBuf};

use io::catalog::{
    EditGraph, ImageDO,
    catalog::{CATALOG_FILE_NAME, CATALOG_FOLDER_NAME, Catalog},
    catalog_error::CatalogError,
};

use crate::{catalog::selection::sync_selection, error::ServiceError, types::SelectionSyncResult};

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

    // TODO: Remove
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

    pub fn root(&self) -> &Path {
        self.catalog.root()
    }

    pub fn cache_dir(&self) -> &Path {
        self.catalog.cache_dir()
    }

    pub fn preview_cache_dir(&self) -> &Path {
        self.catalog.preview_cache_dir()
    }

    pub fn develop_cache_dir(&self) -> &Path {
        self.catalog.develop_cache_dir()
    }

    pub async fn sync_selection(
        &self,
        request_id: u64,
        selected_path: PathBuf,
    ) -> Result<SelectionSyncResult, ServiceError> {
        sync_selection(&self.catalog, request_id, selected_path).await
    }
}
