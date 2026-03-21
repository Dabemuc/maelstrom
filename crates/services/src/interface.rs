use std::sync::Arc;

use crate::{catalog::CatalogService, error::ServiceError};

/// This struct provides access to all the use cases of the backend
#[derive(Debug, Clone)]
pub struct Services {
    pub catalog: Arc<CatalogService>,
}

impl Services {
    pub fn new(catalog: CatalogService) -> Result<Self, ServiceError> {
        Ok(Self {
            catalog: Arc::new(catalog),
        })
    }
}
