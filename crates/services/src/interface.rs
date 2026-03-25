use std::sync::Arc;

use tokio::sync::broadcast;

use crate::catalog::CatalogService;
use crate::error::ServiceError;
use crate::event_bus::EventBus;
use crate::events::ServiceEvent;
use crate::task_manager::TaskManager;

/// This struct provides access to all the use cases of the backend
#[derive(Debug, Clone)]
pub struct Services {
    pub catalog: Arc<CatalogService>,
    pub tasks: Arc<TaskManager>,
    pub(crate) bus: Arc<EventBus>,
}

impl Services {
    pub fn new(catalog: CatalogService) -> Result<Self, ServiceError> {
        let bus = EventBus::new();
        Self::new_with_bus(catalog, bus)
    }

    /// Creates Services with a pre-existing EventBus. Use this when the frontend
    /// creates the bus before services are initialized so the subscription is stable.
    pub fn new_with_bus(catalog: CatalogService, bus: Arc<EventBus>) -> Result<Self, ServiceError> {
        let tasks = TaskManager::new(bus.clone());
        Ok(Self {
            catalog: Arc::new(catalog),
            tasks,
            bus,
        })
    }

    /// Returns a receiver for the service event broadcast channel.
    /// Used by the frontend subscription to listen for async task progress.
    pub fn subscribe(&self) -> broadcast::Receiver<ServiceEvent> {
        self.bus.subscribe()
    }
}
