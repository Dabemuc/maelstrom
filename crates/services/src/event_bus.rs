use std::sync::Arc;

use tokio::sync::broadcast;

use crate::events::ServiceEvent;

#[derive(Debug)]
pub struct EventBus {
    sender: broadcast::Sender<ServiceEvent>,
}

impl EventBus {
    pub fn new() -> Arc<Self> {
        let (sender, _) = broadcast::channel(512);
        Arc::new(Self { sender })
    }

    pub fn emit(&self, event: ServiceEvent) {
        // Dropping the result is intentional: no subscriber is fine (e.g. during startup)
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServiceEvent> {
        self.sender.subscribe()
    }
}
