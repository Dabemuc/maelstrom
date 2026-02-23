use super::turso::TursoDB;
use crate::catalog::catalog_error::CatalogError;
use std::path::{Path, PathBuf};

const CATALOG_VERSION: u16 = 1;

pub struct Catalog {
    db: TursoDB,
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
            Some(v) if v == CATALOG_VERSION => Ok(Self { db }),
            Some(v) => Err(CatalogError::VersionMismatch {
                expected: CATALOG_VERSION,
                found: v,
            }),
            None => Err(CatalogError::MissingVersion),
        }
    }

    /// Creates a new catalog (or opens existing) and initializes the schema.
    /// Sets the version if creating a new catalog.
    pub async fn create(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let path_ref = path.as_ref();

        let path_str = path_ref
            .to_str()
            .ok_or_else(|| CatalogError::InvalidPathEncoding(path_ref.to_path_buf()))?;

        let db = TursoDB::create(path_str).await?;

        let existing_version = db.get_version().await?;
        match existing_version {
            Some(v) if v == CATALOG_VERSION => Ok(Self { db }),
            Some(v) => Err(CatalogError::VersionMismatch {
                expected: CATALOG_VERSION,
                found: v,
            }),
            None => {
                db.set_version(CATALOG_VERSION).await?;
                Ok(Self { db })
            }
        }
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
}
