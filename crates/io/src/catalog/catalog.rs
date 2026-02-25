use super::turso::TursoDB;
use crate::catalog::catalog_error::CatalogError;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const CATALOG_VERSION: u16 = 1;
pub const CATALOG_FILE_NAME: &str = "catalog.mcat";
pub const CATALOG_FOLDER_NAME: &str = "maelstrom_catalog";
pub const CACHE_DIR_NAME: &str = "cache";

#[derive(Clone)]
pub struct Catalog {
    db: Arc<TursoDB>,
    root: PathBuf,
    cache_dir: PathBuf,
}

impl Catalog {
    /// Opens an existing catalog directory.
    /// Verifies:
    /// - directory exists
    /// - catalog.mcat exists
    /// - cache/ exists
    /// - version matches
    pub async fn load(folder: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let folder_ref = folder.as_ref();

        if !folder_ref.is_dir() {
            return Err(CatalogError::InvalidPathEncoding(folder_ref.to_path_buf()));
        }

        let catalog_path = folder_ref.join(CATALOG_FILE_NAME);
        let cache_dir = folder_ref.join(CACHE_DIR_NAME);

        if !catalog_path.is_file() {
            return Err(CatalogError::MissingCatalogFile(catalog_path));
        }

        if !cache_dir.is_dir() {
            return Err(CatalogError::MissingCacheDirectory(cache_dir));
        }

        let path_str = catalog_path
            .to_str()
            .ok_or_else(|| CatalogError::InvalidPathEncoding(catalog_path.clone()))?;

        let db = TursoDB::open(path_str)
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?;

        let version = db
            .get_version()
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?;

        match version {
            Some(v) if v == CATALOG_VERSION => Ok(Self {
                db: Arc::new(db),
                root: folder_ref.to_path_buf(),
                cache_dir,
            }),
            Some(v) => Err(CatalogError::VersionMismatch {
                expected: CATALOG_VERSION,
                found: v,
            }),
            None => Err(CatalogError::MissingVersion),
        }
    }

    /// Creates a new catalog directory structure:
    ///
    /// <base>/maelstrom_catalog/
    ///   catalog.mcat
    ///   cache/
    pub async fn create(base_folder: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let base_ref = base_folder.as_ref();

        if !base_ref.is_dir() {
            return Err(CatalogError::InvalidPathEncoding(base_ref.to_path_buf()));
        }

        // Create catalog folder inside selected base folder
        let root = base_ref.join(CATALOG_FOLDER_NAME);

        if root.exists() {
            return Err(CatalogError::AlreadyExists(root));
        }

        // Create catalog root directory
        fs::create_dir(&root).map_err(|e| CatalogError::FileSystem(e.to_string()))?;

        let catalog_path = root.join(CATALOG_FILE_NAME);
        let cache_dir = root.join(CACHE_DIR_NAME);

        // Create cache directory
        fs::create_dir(&cache_dir).map_err(|e| CatalogError::FileSystem(e.to_string()))?;

        let path_str = catalog_path
            .to_str()
            .ok_or_else(|| CatalogError::InvalidPathEncoding(catalog_path.clone()))?;

        let db = TursoDB::create(path_str)
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?;

        db.set_version(CATALOG_VERSION)
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?;

        Ok(Self {
            db: Arc::new(db),
            root,
            cache_dir,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Imports a directory path into the catalog.
    pub async fn import_directory(&self, path: impl AsRef<Path>) -> Result<(), CatalogError> {
        let path_ref = path.as_ref();

        let path_str = path_ref
            .to_str()
            .ok_or_else(|| CatalogError::InvalidPathEncoding(path_ref.to_path_buf()))?;

        self.db.add_imported_path(path_str).await?;
        Ok(())
    }

    /// Retrieves all imported directory paths.
    pub async fn get_imported_directories(&self) -> Result<Vec<PathBuf>, CatalogError> {
        let paths = self.db.get_imported_paths().await?;
        Ok(paths.into_iter().map(PathBuf::from).collect())
    }

    pub async fn image_exists(&self, content_hash: &str) -> Result<bool, CatalogError> {
        let res = self.db.image_exists(content_hash).await?;
        Ok(res)
    }

    pub async fn add_image(
        &self,
        content_hash: &str,
        path: impl AsRef<Path>,
    ) -> Result<(), CatalogError> {
        let path_ref = path.as_ref();

        let path_str = path_ref
            .to_str()
            .ok_or_else(|| CatalogError::InvalidPathEncoding(path_ref.to_path_buf()))?;

        self.db.add_image(content_hash, path_str).await?;
        Ok(())
    }

    pub async fn get_all_hashes_for_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<String>, CatalogError> {
        let path_ref = path.as_ref();

        let path_str = path_ref
            .to_str()
            .ok_or_else(|| CatalogError::InvalidPathEncoding(path_ref.to_path_buf()))?;

        let hashes = self.db.get_image_hashes_by_path(path_str).await?;
        Ok(hashes)
    }

    /// Prints catalog metadata: version and imported directories.
    /// Returns an error if the database cannot be accessed.
    pub async fn print_metadata(&self) -> Result<(), CatalogError> {
        let version = self
            .db
            .get_version()
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?
            .ok_or(CatalogError::MissingVersion)?;

        let dirs = self.get_imported_directories().await?;

        println!("Catalog root: {:?}", self.root);
        println!("Catalog version: {}", version);
        println!("Imported directories:");
        for dir in dirs {
            println!(" - {:?}", dir);
        }

        Ok(())
    }
}

impl std::fmt::Debug for Catalog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Catalog")
            .field("root", &self.root)
            .field("cache_dir", &self.cache_dir)
            .finish()
    }
}
