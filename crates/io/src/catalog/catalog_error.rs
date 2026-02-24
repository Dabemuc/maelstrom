use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum CatalogError {
    #[error("Invalid path encoding: {0:?}")]
    InvalidPathEncoding(PathBuf),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Catalog version mismatch. Expected {expected}, found {found}")]
    VersionMismatch { expected: u16, found: u16 },

    #[error("Catalog version missing in existing file")]
    MissingVersion,

    #[error("Catalog already exists at {0:?}")]
    AlreadyExists(PathBuf),

    #[error("Catalog folder does not contain catalog file at {0:?}")]
    MissingCatalogFile(PathBuf),

    #[error("Catalog folder does not contain cache folder at {0:?}")]
    MissingCacheDirectory(PathBuf),

    #[error("Filesystem error concerning catalog: {0:?}")]
    FileSystem(String),
}

// Convert boxed dynamic errors
impl From<Box<dyn std::error::Error + Send + Sync>> for CatalogError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        CatalogError::Database(err.to_string())
    }
}

// Convert turso::Error directly
impl From<turso::Error> for CatalogError {
    fn from(err: turso::Error) -> Self {
        CatalogError::Database(err.to_string())
    }
}
