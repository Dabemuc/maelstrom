use io::catalog::catalog_error::CatalogError;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum StateError {
    #[error("Catalog error")]
    CatalogError(#[from] CatalogError),

    #[error("Other error: {0}")]
    BoxedError(String),
}

// Convert boxed dynamic errors
impl From<Box<dyn std::error::Error>> for StateError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Self::BoxedError(err.to_string())
    }
}
