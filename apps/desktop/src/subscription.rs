use std::hash::Hash;
use std::sync::Arc;

use iced::Subscription;
use iced::advanced::subscription::{EventStream, Hasher, Recipe, from_recipe};
use iced::futures::stream::BoxStream;
use iced::futures::stream;
use services::event_bus::EventBus;
use services::events::ServiceEvent;
use tokio::sync::broadcast;

use crate::message::Message;

struct ServiceEventRecipe {
    bus: Arc<EventBus>,
}

impl Recipe for ServiceEventRecipe {
    type Output = Message;

    fn hash(&self, state: &mut Hasher) {
        // One stable subscription for the app lifetime
        std::any::TypeId::of::<ServiceEventRecipe>().hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Message> {
        let rx = self.bus.subscribe();
        Box::pin(stream::unfold(rx, |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(event) => return Some((map_event(event), rx)),
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("[ServiceEvents] Lagged, dropped {n} events");
                        // continue receiving — don't drop the subscription
                    }
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        }))
    }
}

fn map_event(event: ServiceEvent) -> Message {
    match event {
        ServiceEvent::PreviewGenerated { result, .. } => Message::PreviewGenerated(result),
        ServiceEvent::ImportCompleted { payload, .. } => Message::ImportCompleted(payload),
        ServiceEvent::SyncCompleted { result, .. } => Message::SelectionSynced(Ok(result)),
    }
}

pub fn service_events(bus: &Arc<EventBus>) -> Subscription<Message> {
    from_recipe(ServiceEventRecipe { bus: bus.clone() })
}
