pub mod catalog;
pub mod helpers;
pub mod import;
pub mod navigator;
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
        Message::CreateCatalog => catalog::handle_create_catalog(app),
        Message::SelectCatalog => catalog::handle_select_catalog(app),
        Message::CatalogLoadAttempted(result) => {
            catalog::handle_catalog_load_attempted(app, result)
        }
        Message::CatalogLoaded => catalog::handle_catalog_loaded(app),
        Message::NavigatorCollapseAll => navigator::handle_navigator_collapse_all(app),
        Message::ImportDirectory => import::handle_import_directory(app),
        Message::LoadImportedDirectories => import::handle_load_imported_directories(app),
        Message::ImportedDirectoriesLoadAttempted(result) => {
            import::handle_imported_directories_load_attempted(app, result)
        }
        Message::ErrorMessage(msg) => workspace::handle_error_message(app, msg),
        Message::ToggleDirectory(path) => navigator::handle_toggle_directory(app, path),
        Message::SelectDirectory(path) => navigator::handle_select_directory(app, path),
        Message::OpenRootContextMenu(path) => navigator::handle_open_root_context_menu(app, path),
        Message::CloseRootContextMenu => navigator::handle_close_root_context_menu(app),
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
    }
}
