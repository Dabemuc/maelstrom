use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("Invalid path encoding: {0:?}")]
    InvalidPathEncoding(PathBuf),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Catalog version mismatch. Expected {expected}, found {found}")]
    VersionMismatch { expected: u16, found: u16 },

    #[error("Catalog version missing in existing file")]
    MissingVersion,
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
