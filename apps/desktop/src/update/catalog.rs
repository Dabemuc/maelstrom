use iced::Task;
use io::catalog::catalog::Catalog;
use io::catalog::catalog_error::CatalogError;
use rfd::FileDialog;

use crate::app::App;
use crate::message::Message;
use crate::state::ViewMode;

pub fn handle_create_catalog(_app: &mut App) -> Task<Message> {
    if let Some(path) = FileDialog::new().pick_folder() {
        Task::perform(Catalog::create(path), Message::CatalogLoadAttempted)
    } else {
        println!("FileDialog canceled");
        Task::none()
    }
}

pub fn handle_select_catalog(_app: &mut App) -> Task<Message> {
    if let Some(path) = FileDialog::new().pick_folder() {
        Task::perform(Catalog::load(path), Message::CatalogLoadAttempted)
    } else {
        println!("FileDialog canceled");
        Task::none()
    }
}

pub fn handle_catalog_load_attempted(
    app: &mut App,
    result: Result<Catalog, CatalogError>,
) -> Task<Message> {
    match result {
        Ok(catalog) => {
            crate::app::startup_log("CatalogLoadAttempted: success");
            app.catalog = Some(catalog);
            Task::perform(async {}, |_| Message::CatalogLoaded)
        }
        Err(e) => {
            crate::app::startup_log("CatalogLoadAttempted: error");
            eprintln!("Error loading catalog: {}", e);
            Task::none()
        }
    }
}

pub fn handle_catalog_loaded(app: &mut App) -> Task<Message> {
    crate::app::startup_log("CatalogLoaded message received");
    app.view_mode = ViewMode::Library;
    crate::app::startup_log("Dispatching LoadImportedDirectories");
    Task::perform(async {}, |_| Message::LoadImportedDirectories)
}
