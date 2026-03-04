use std::path::PathBuf;

use iced::Task;

use crate::app::App;
use crate::message::Message;

pub fn handle_navigator_collapse_all(app: &mut App) -> Task<Message> {
    app.navigator_state.expanded.clear();
    Task::none()
}

pub fn handle_toggle_directory(app: &mut App, path: PathBuf) -> Task<Message> {
    app.navigator_state.context_menu_open = false;
    app.navigator_state.context_menu_root = None;

    if app.navigator_state.expanded.contains(&path) {
        app.navigator_state.expanded.remove(&path);
    } else {
        app.navigator_state.expanded.insert(path);
    }
    Task::none()
}

pub fn handle_select_directory(app: &mut App, path: PathBuf) -> Task<Message> {
    app.navigator_state.context_menu_open = false;
    app.navigator_state.context_menu_root = None;

    if app.navigator_state.selected.as_ref() == Some(&path) {
        app.navigator_state.selected = None;
        app.workspace_state.previews.clear();
        app.workspace_state.sorted_preview_keys.clear();
        app.selection_request_seq = app.selection_request_seq.wrapping_add(1);
        app.active_selection_request_id = None;
        return Task::none();
    }

    app.navigator_state.selected = Some(path.clone());
    app.selection_request_seq = app.selection_request_seq.wrapping_add(1);
    let request_id = app.selection_request_seq;
    app.active_selection_request_id = Some(request_id);
    crate::update::helpers::refresh_selected_previews_from_cache(app);

    let Some(catalog) = app.catalog.clone() else {
        return Task::none();
    };

    Task::perform(
        async move {
            let selected_path = path.clone();
            let image_dos = catalog.get_all_image_dos_for_path(&selected_path).await?;
            Ok((request_id, selected_path, image_dos))
        },
        Message::SelectionCatalogLoaded,
    )
}

pub fn handle_open_root_context_menu(app: &mut App, path: PathBuf) -> Task<Message> {
    app.navigator_state.context_menu_root = Some(path);
    app.navigator_state.context_menu_open = true;
    Task::none()
}

pub fn handle_close_root_context_menu(app: &mut App) -> Task<Message> {
    app.navigator_state.context_menu_open = false;
    app.navigator_state.context_menu_root = None;
    Task::none()
}
