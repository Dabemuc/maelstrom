use iced::Task;
use services::{error::ServiceError, interface::Services};

use crate::{app::App, message::Message};

pub fn handle_services_initialized(
    app: &mut App,
    result: Result<Services, ServiceError>,
) -> Task<Message> {
    match result {
        Ok(services) => {
            app.services = Some(services.clone());
            println!("Services initialized: {:#?}", services.clone());

            // Setup UI
            app.view_mode = crate::state::ViewMode::Library;

            Task::perform(async {}, |_| Message::LoadManagedDirectories)
        }
        Err(e) => {
            println!("Error while initializing services: {}", e);
            Task::none()
        }
    }
}
