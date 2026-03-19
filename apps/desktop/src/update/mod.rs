pub mod develop;
pub mod directories;
pub mod helpers;
pub mod import;
pub mod managed;
pub mod pane_grid;
pub mod preview;
pub mod selection;
pub mod services;
pub mod sidebar;
pub mod workspace;

use iced::Task;

use crate::app::App;
use crate::message::Message;

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::LeftSidebarClicked(mode) => sidebar::handle_left_sidebar_clicked(app, mode),
        Message::RightSidebarClicked(mode) => sidebar::handle_right_sidebar_clicked(app, mode),
        Message::PaneResized(event) => pane_grid::handle_pane_resized(app, event),
        Message::ServicesInitialized(result) => services::handle_services_initialized(app, result),
        Message::DirectoriesCollapseAll => directories::handle_directories_collapse_all(app),
        Message::AddManagedDirectory => managed::handle_add_managed_directory(app),
        Message::LoadManagedDirectories => managed::handle_load_managed_directories(app),
        Message::ManagedDirectoriesLoadAttempted(result) => {
            managed::handle_managed_directories_load_attempted(app, result)
        }
        Message::Notification(msg) => workspace::handle_error_message(app, msg),
        Message::ToggleDirectory(path) => directories::handle_toggle_directory(app, path),
        Message::SelectDirectory(path) => directories::handle_select_directory(app, path),
        Message::OpenRootContextMenu(path) => directories::handle_open_root_context_menu(app, path),
        Message::CloseRootContextMenu => directories::handle_close_root_context_menu(app),
        Message::RefreshManagedRoot(root) => workspace::handle_refresh_managed_root(app, root),
        Message::ImportFotos(root) => import::handle_import_fotos_into_managed_root(app, root),
        Message::WorkspaceRootScanned((root, scan_result)) => {
            workspace::handle_workspace_root_scanned(app, root, scan_result)
        }
        Message::SelectionSynced(result) => selection::handle_selection_synced(app, result),
        Message::PreviewGenerated(result) => preview::handle_preview_generated(app, result),
        Message::ImportCompleted(payload) => import::handle_import_completed(app, payload),
        Message::SortingOptionSelected(option) => {
            workspace::handle_sorting_option_selected(app, option)
        }
        Message::SortingDirectionToggled => workspace::handle_sorting_direction_toggled(app),
        Message::PreviewDoubleClicked(hash) => workspace::handle_preview_selected(app, hash).chain(
            workspace::handle_view_mode_selected(app, crate::state::ViewMode::Develop),
        ),
        Message::ViewModeSelected(mode) => workspace::handle_view_mode_selected(app, mode),
        Message::PreviewSelected(hash) => workspace::handle_preview_selected(app, hash),
        Message::DevelopStateLoaded(result) => develop::handle_develop_state_loaded(app, result),
        Message::ImageDeveloped(linear_image) => develop::handle_image_developed(app, linear_image),
        Message::DevelopZoomSet(zoom) => develop::handle_develop_zoom_set(app, zoom),
        Message::DevelopZoomBy(factor) => develop::handle_develop_zoom_by(app, factor),
        Message::DevelopZoomSetPan { zoom, pan } => {
            develop::handle_develop_zoom_set_pan(app, zoom, pan)
        }
        Message::DevelopFitToScreen => develop::handle_develop_fit_to_screen(app),
        Message::DevelopPanBy { delta } => develop::handle_develop_pan_by(app, delta),
        Message::DevelopParamChanged { kind, name, value } => {
            develop::handle_develop_param_changed(app, kind, name, value)
        }
        Message::DevelopParamInputChanged { kind, name, value } => {
            develop::handle_develop_param_input_changed(app, kind, name, value)
        }
        Message::DevelopSaveRequested => develop::handle_develop_save_requested(app),
        Message::DevelopSaveCompleted(result) => {
            develop::handle_develop_save_completed(app, result)
        }
        Message::DevelopExportRequested => develop::handle_develop_export_requested(app),
    }
}
