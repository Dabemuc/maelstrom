use graph::node::Backend;
use iced::{Task, widget::image::Handle};

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
            app.develop_state = Some(state.clone());

            Task::perform(
                async move {
                    let developed_linear_image = state
                        .edit_graph
                        .compile()
                        .execute(state.original_linear_image, Backend::Cpu);

                    Handle::from_rgba(
                        developed_linear_image.width,
                        developed_linear_image.height,
                        developed_linear_image.to_pixels(),
                    )
                },
                Message::ImageDeveloped,
            )
        }
        Err(e) => {
            println!("Error loading develop state: {:#?}", e);
            Task::none()
        }
    }
}

pub fn handle_image_developed(app: &mut App, handle: Handle) -> Task<Message> {
    println!("Image developed succesfully");
    if let Some(develop_state) = app.develop_state.as_mut() {
        develop_state.developed_handle = Some(handle);
    } else {
        println!(
            "Failed to store developed image in state. No develop state. This should not happen!"
        )
    }

    Task::none()
}
