use std::path::PathBuf;

use iced::Task;
use iced::futures::channel::oneshot;
use io::catalog::catalog_error::CatalogError;
use io::image_files::helpers::scan_folder_images;

use crate::app::App;
use crate::message::Message;

pub fn handle_add_managed_directory(app: &mut App) -> Task<Message> {
    let Some(catalog) = app.catalog.clone() else {
        println!("Cannot add managed directory: no catalog loaded");
        return Task::none();
    };

    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        let catalog_clone = catalog.clone();
        Task::perform(
            async move { catalog_clone.add_managed_directory(path.clone()).await },
            |res| match res {
                Ok(_) => Message::LoadManagedDirectories,
                Err(_e) => Message::Notification("Failed to add managed directory".into()),
            },
        )
    } else {
        println!("FileDialog canceled");
        Task::none()
    }
}

pub fn handle_load_managed_directories(app: &mut App) -> Task<Message> {
    crate::app::startup_log("LoadManagedDirectories started");
    if let Some(catalog) = &app.catalog {
        let catalog_clone = catalog.clone();
        Task::perform(
            async move { catalog_clone.get_managed_directories().await },
            Message::ManagedDirectoriesLoadAttempted,
        )
    } else {
        crate::app::startup_log("LoadManagedDirectories skipped (no catalog)");
        Task::none()
    }
}

pub fn handle_managed_directories_load_attempted(
    app: &mut App,
    result: Result<Vec<PathBuf>, CatalogError>,
) -> Task<Message> {
    match result {
        Ok(paths) => {
            crate::app::startup_log(&format!(
                "ManagedDirectoriesLoadAttempted: success ({} roots)",
                paths.len()
            ));
            app.managed_dirs = paths.clone();

            app.workspace_state.model.clear();
            app.workspace_state.model.root_folders = paths.clone();
            app.workspace_state.preview_cache.clear();
            app.workspace_state.previews.clear();
            app.workspace_state.sorted_preview_keys.clear();
            app.workspace_state.roots_scanning = paths.iter().cloned().collect();

            crate::app::startup_log("Dispatching root scan tasks");

            let scan_tasks: Vec<Task<Message>> = paths
                .iter()
                .map(|root| {
                    let root = root.clone();
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
            crate::app::startup_log("ManagedDirectoriesLoadAttempted: error");
            println!(
                "Error while loading managed directories from catalog: {0:?}",
                e
            );
            Task::none()
        }
    }
}
