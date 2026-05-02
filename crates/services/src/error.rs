use io::catalog::catalog_error::CatalogError;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ServiceError {
    #[error("Catalog error: {0}")]
    Catalog(#[from] CatalogError),
}
