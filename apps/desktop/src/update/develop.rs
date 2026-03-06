use std::sync::Arc;

use graph::node::Backend;
use iced::Task;
use maelstrom_image::linear_image::LinearImage;

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
                    state
                        .edit_graph
                        .compile()
                        .execute(state.original_linear_image, Backend::Cpu)
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

pub fn handle_image_developed(app: &mut App, developed_linear_image: LinearImage) -> Task<Message> {
    println!("Image developed succesfully");
    if let Some(develop_state) = app.develop_state.as_mut() {
        develop_state.developed_linear_image = Some(Arc::new(developed_linear_image));
    } else {
        println!(
            "Failed to store developed image in state. No develop state. This should not happen!"
        )
    }

    Task::none()
}

pub fn handle_develop_zoom_set(app: &mut App, zoom: f32) -> Task<Message> {
    if let Some(state) = app.develop_state.as_mut() {
        state.zoom = zoom.clamp(0.05, 64.0);
        state.zoom_mode = crate::state::develop::ZoomMode::Manual;
    }

    Task::none()
}

pub fn handle_develop_zoom_by(app: &mut App, factor: f32) -> Task<Message> {
    if let Some(state) = app.develop_state.as_mut() {
        let new_zoom = (state.zoom * factor).clamp(0.05, 64.0);
        state.zoom = new_zoom;
        state.zoom_mode = crate::state::develop::ZoomMode::Manual;
    }

    Task::none()
}

pub fn handle_develop_zoom_set_pan(app: &mut App, zoom: f32, pan: [f32; 2]) -> Task<Message> {
    if let Some(state) = app.develop_state.as_mut() {
        state.zoom = zoom.clamp(0.05, 64.0);
        state.pan = pan;
        state.zoom_mode = crate::state::develop::ZoomMode::Manual;
    }

    Task::none()
}

pub fn handle_develop_fit_to_screen(app: &mut App) -> Task<Message> {
    if let Some(state) = app.develop_state.as_mut() {
        state.fit_request = state.fit_request.wrapping_add(1);
        state.pan = [0.0, 0.0];
        state.zoom_mode = crate::state::develop::ZoomMode::FitOnce;
    }

    Task::none()
}

pub fn handle_develop_pan_by(app: &mut App, delta: [f32; 2]) -> Task<Message> {
    if let Some(state) = app.develop_state.as_mut() {
        state.pan[0] += delta[0];
        state.pan[1] += delta[1];
    }

    Task::none()
}
