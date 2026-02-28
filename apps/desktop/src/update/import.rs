use std::path::PathBuf;

use iced::Task;
use iced::futures::channel::oneshot;
use io::catalog::catalog_error::CatalogError;
use io::image_files::helpers::scan_folder_images;

use crate::app::App;
use crate::message::Message;

pub fn handle_import_directory(app: &mut App) -> Task<Message> {
    let Some(catalog) = app.catalog.clone() else {
        println!("Cannot import directory: no catalog loaded");
        return Task::none();
    };

    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        let catalog_clone = catalog.clone();
        Task::perform(
            async move { catalog_clone.import_directory(path.clone()).await },
            |res| match res {
                Ok(_) => Message::LoadImportedDirectories,
                Err(_e) => Message::ErrorMessage("Failed to import directory".into()),
            },
        )
    } else {
        println!("FileDialog canceled");
        Task::none()
    }
}

pub fn handle_load_imported_directories(app: &mut App) -> Task<Message> {
    crate::app::startup_log("LoadImportedDirectories started");
    if let Some(catalog) = &app.catalog {
        let catalog_clone = catalog.clone();
        Task::perform(
            async move { catalog_clone.get_imported_directories().await },
            Message::ImportedDirectoriesLoadAttempted,
        )
    } else {
        crate::app::startup_log("LoadImportedDirectories skipped (no catalog)");
        Task::none()
    }
}

pub fn handle_imported_directories_load_attempted(
    app: &mut App,
    result: Result<Vec<PathBuf>, CatalogError>,
) -> Task<Message> {
    match result {
        Ok(paths) => {
            crate::app::startup_log(&format!(
                "ImportedDirectoriesLoadAttempted: success ({} roots)",
                paths.len()
            ));
            app.imported_dirs = paths.clone();

            app.workspace_state.model.clear();
            app.workspace_state.model.root_folders = paths.clone();
            app.workspace_state.preview_cache.clear();
            app.workspace_state.previews.clear();
            app.workspace_state.roots_scanning = paths.iter().cloned().collect();

            crate::app::startup_log("Dispatching root scan tasks");

            let scan_tasks: Vec<Task<Message>> = paths
                .iter()
                .cloned()
                .map(|root| {
                    Task::perform(
                        async move {
                            let (tx, rx) = oneshot::channel();
                            std::thread::spawn(move || {
                                let scan_result = scan_folder_images(root.clone());
                                let _ = tx.send((root, scan_result));
                            });

                            rx.await.expect("Thread panicked or channel closed")
                        },
                        Message::WorkspaceRootScanned,
                    )
                })
                .collect();

            Task::batch(scan_tasks)
        }
        Err(e) => {
            crate::app::startup_log("ImportedDirectoriesLoadAttempted: error");
            println!(
                "Error while loading imported directories from catalog: {0:?}",
                e
            );
            Task::none()
        }
    }
}
