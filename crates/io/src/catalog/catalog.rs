use super::turso::TursoDB;
use std::path::{Path, PathBuf};

static CATALOG_VERSION: u16  = 0;

pub struct Catalog {
    db: TursoDB,
}

impl Catalog {
    /// Opens an existing catalog. Fails if the file does not exist.
    pub async fn load(
        path: impl AsRef<Path>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| "Invalid path encoding".to_string())?;
        let db = TursoDB::open(path_str).await?;
        Ok(Self { db })
    }

    /// Creates a new catalog (or opens existing) and initializes the schema.
    pub async fn create(
        path: impl AsRef<Path>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| "Invalid path encoding".to_string())?;
        let db = TursoDB::create(path_str).await?;
        Ok(Self { db })
    }

    /// Imports a directory path into the catalog.
    pub async fn import_directory(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| "Invalid path encoding".to_string())?;
        self.db.add_imported_path(path_str).await?;
        Ok(())
    }

    /// Retrieves all imported directory paths.
    pub async fn get_imported_directories(
        &self,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        let paths = self.db.get_imported_paths().await?;
        Ok(paths.into_iter().map(PathBuf::from).collect())
    }
}
