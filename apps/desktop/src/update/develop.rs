use std::sync::Arc;

use graph::node::Backend;
use iced::Task;
use io::catalog::edit_graph::{EditNodeKind, NodeParameters, ParamType, ParamValue};
use maelstrom_image::linear_image::LinearImage;

use crate::{
    app::App,
    message::Message,
    state::{develop::{DevelopState, ParamKey}, state_error::StateError},
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

pub fn handle_develop_param_changed(
    app: &mut App,
    kind: EditNodeKind,
    name: String,
    value: ParamValue,
) -> Task<Message> {
    let Some(state) = app.develop_state.as_mut() else {
        return Task::none();
    };

    let changed = apply_param_change(state, kind, name.as_str(), value.clone());

    if changed {
        let key = ParamKey { kind, name };
        match value {
            ParamValue::Float(v) => {
                state.param_inputs.insert(key, format!("{:.2}", v));
            }
            ParamValue::Int(v) => {
                state.param_inputs.insert(key, v.to_string());
            }
            ParamValue::Bool(_) => {}
        }
    } else {
        return Task::none();
    }

    let graph = state.edit_graph.clone();
    let original_linear_image = state.original_linear_image.clone();

    Task::perform(
        async move {
            graph
                .compile()
                .execute(original_linear_image, Backend::Cpu)
        },
        Message::ImageDeveloped,
    )
}

pub fn handle_develop_param_input_changed(
    app: &mut App,
    kind: EditNodeKind,
    name: String,
    value: String,
) -> Task<Message> {
    let Some(state) = app.develop_state.as_mut() else {
        return Task::none();
    };

    let key = ParamKey {
        kind,
        name: name.clone(),
    };
    state.param_inputs.insert(key.clone(), value.clone());

    let spec = kind
        .default_node()
        .param_specs()
        .iter()
        .find(|spec| spec.name == name);

    let Some(spec) = spec else {
        return Task::none();
    };

    let trimmed = value.trim();

    let applied = match spec.ty {
        ParamType::Float { min, max, .. } => trimmed
            .parse::<f32>()
            .ok()
            .map(|parsed| {
                let clamped = parsed.clamp(min, max);
                let changed = apply_param_change(
                    state,
                    kind,
                    name.as_str(),
                    ParamValue::Float(clamped),
                );

                if (clamped - parsed).abs() > f32::EPSILON {
                    state
                        .param_inputs
                        .insert(key, format!("{:.2}", clamped));
                }

                changed
            })
            .unwrap_or(false),
        ParamType::Int { min, max } => trimmed
            .parse::<i32>()
            .ok()
            .map(|parsed| {
                let clamped = parsed.clamp(min, max);
                let changed = apply_param_change(
                    state,
                    kind,
                    name.as_str(),
                    ParamValue::Int(clamped),
                );

                if clamped != parsed {
                    state.param_inputs.insert(key, clamped.to_string());
                }

                changed
            })
            .unwrap_or(false),
        ParamType::Bool => false,
    };

    if !applied {
        return Task::none();
    }

    let graph = state.edit_graph.clone();
    let original_linear_image = state.original_linear_image.clone();

    Task::perform(
        async move {
            graph
                .compile()
                .execute(original_linear_image, Backend::Cpu)
        },
        Message::ImageDeveloped,
    )
}

fn apply_param_change(
    state: &mut DevelopState,
    kind: EditNodeKind,
    name: &str,
    value: ParamValue,
) -> bool {
    let existing_index = state
        .edit_graph
        .nodes
        .iter()
        .position(|node| node.kind() == kind);

    if let Some(index) = existing_index {
        let before = state.edit_graph.nodes[index].clone();
        let node = &mut state.edit_graph.nodes[index];
        node.set_param(name, value);

        if node.is_default() {
            state.edit_graph.nodes.remove(index);
            return true;
        }

        return before != *node;
    }

    let mut node = kind.default_node();
    node.set_param(name, value);

    if node.is_default() {
        return false;
    }

    state.edit_graph.nodes.push(node);
    true
}
