pub mod catalog;
pub mod develop;
pub mod helpers;
pub mod import;
pub mod directories;
pub mod pane_grid;
pub mod preview;
pub mod selection;
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
        Message::CreateCatalog => catalog::handle_create_catalog(app),
        Message::SelectCatalog => catalog::handle_select_catalog(app),
        Message::CatalogLoadAttempted(result) => {
            catalog::handle_catalog_load_attempted(app, result)
        }
        Message::CatalogLoaded => catalog::handle_catalog_loaded(app),
        Message::DirectoriesCollapseAll => directories::handle_directories_collapse_all(app),
        Message::ImportDirectory => import::handle_import_directory(app),
        Message::LoadImportedDirectories => import::handle_load_imported_directories(app),
        Message::ImportedDirectoriesLoadAttempted(result) => {
            import::handle_imported_directories_load_attempted(app, result)
        }
        Message::ErrorMessage(msg) => workspace::handle_error_message(app, msg),
        Message::ToggleDirectory(path) => directories::handle_toggle_directory(app, path),
        Message::SelectDirectory(path) => directories::handle_select_directory(app, path),
        Message::OpenRootContextMenu(path) => directories::handle_open_root_context_menu(app, path),
        Message::CloseRootContextMenu => directories::handle_close_root_context_menu(app),
        Message::RefreshImportedRoot(root) => workspace::handle_refresh_imported_root(app, root),
        Message::WorkspaceRootScanned((root, scan_result)) => {
            workspace::handle_workspace_root_scanned(app, root, scan_result)
        }
        Message::SelectionCatalogLoaded(result) => {
            selection::handle_selection_catalog_loaded(app, result)
        }
        Message::SelectionDiffComputed(diff_data) => {
            selection::handle_selection_diff_computed(app, diff_data)
        }
        Message::PreviewDataLoadedForImage(preview) => {
            preview::handle_preview_data_loaded_for_image(app, preview)
        }
        Message::PreviewGenerated(result) => preview::handle_preview_generated(app, result),
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
        Message::DevelopZoomSetPan { zoom, pan } => develop::handle_develop_zoom_set_pan(app, zoom, pan),
        Message::DevelopFitToScreen => develop::handle_develop_fit_to_screen(app),
        Message::DevelopPanBy { delta } => develop::handle_develop_pan_by(app, delta),
        Message::DevelopParamChanged { kind, name, value } => {
            develop::handle_develop_param_changed(app, kind, name, value)
        }
        Message::DevelopParamInputChanged { kind, name, value } => {
            develop::handle_develop_param_input_changed(app, kind, name, value)
        }
    }
}
