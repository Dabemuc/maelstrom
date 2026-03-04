use iced::Task;

use crate::{
    app::App,
    message::Message,
    state::{develop::DevelopState, state_error::StateError},
};

pub fn handle_develop_state_loaded(
    app: &mut App,
    result: Result<DevelopState, StateError>,
) -> Task<Message> {
    match result {
        Ok(state) => {
            println!("Develop state succesfully loaded");
            app.develop_state = Some(state)
        }
        Err(e) => println!("Error loading develop state: {:#?}", e),
    }

    Task::none()
}
