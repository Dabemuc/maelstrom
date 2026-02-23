use super::turso::TursoDB;
use crate::catalog::catalog_error::CatalogError;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const CATALOG_VERSION: u16 = 1;

#[derive(Clone)]
pub struct Catalog {
    db: Arc<TursoDB>,
}

impl Catalog {
    /// Opens an existing catalog. Fails if the file does not exist or if the version does not match.
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let path_ref = path.as_ref();

        let path_str = path_ref
            .to_str()
            .ok_or_else(|| CatalogError::InvalidPathEncoding(path_ref.to_path_buf()))?;

        let db = TursoDB::open(path_str)
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?;

        let version = db
            .get_version()
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?;

        match version {
            Some(v) if v == CATALOG_VERSION => Ok(Self { db: Arc::new(db) }),
            Some(v) => Err(CatalogError::VersionMismatch {
                expected: CATALOG_VERSION,
                found: v,
            }),
            None => Err(CatalogError::MissingVersion),
        }
    }

    /// Creates a new catalog, initializes the schema and sets the version.
    pub async fn create(folder: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let folder_ref = folder.as_ref();

        // Ensure folder exists and is directory
        if !folder_ref.is_dir() {
            return Err(CatalogError::InvalidPathEncoding(folder_ref.to_path_buf()));
        }

        // Build full file path: <folder>/catalog.mcat
        let catalog_path: PathBuf = folder_ref.join("catalog.mcat");

        // Fail if already exists
        if catalog_path.exists() {
            return Err(CatalogError::AlreadyExists(catalog_path));
        }

        let path_str = catalog_path
            .to_str()
            .ok_or_else(|| CatalogError::InvalidPathEncoding(catalog_path.clone()))?;

        // Create database file
        let db = TursoDB::create(path_str)
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?;

        // Set version immediately (new DB should not have one)
        db.set_version(CATALOG_VERSION)
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?;

        Ok(Self { db: Arc::new(db) })
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

    /// Prints catalog metadata: version and imported directories.
    /// Returns an error if the database cannot be accessed.
    pub async fn print_metadata(&self) -> Result<(), CatalogError> {
        // Get version
        let version = self
            .db
            .get_version()
            .await
            .map_err(|e| CatalogError::Database(e.to_string()))?
            .ok_or(CatalogError::MissingVersion)?;

        // Get imported directories
        let dirs = self.get_imported_directories().await?;

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
        f.debug_struct("Catalog").finish()
    }
}
