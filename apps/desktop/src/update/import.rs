use std::path::PathBuf;

use iced::Task;
use iced::futures::channel::oneshot;
use io::image_files::helpers::scan_folder_images;
use services::types::ImportCompletedPayload;

use crate::update::workspace::handle_error_message;
use crate::{app::App, message::Message};

pub fn handle_import_fotos_into_managed_root(app: &mut App, root: PathBuf) -> Task<Message> {
    let Some(services) = app.services.clone() else {
        println!("Cannot import: no services loaded");
        return Task::none();
    };

    let Some(path) = rfd::FileDialog::new().pick_folder() else {
        println!("FileDialog canceled");
        return Task::none();
    };

    services.catalog.spawn_import_with_previews(
        services::ImportStrategy::DefaultByDate,
        path,
        root,
    );

    Task::none()
}

pub fn handle_import_completed(app: &mut App, payload: ImportCompletedPayload) -> Task<Message> {
    let summary = payload.summary.clone();
    let root = payload.root.clone();

    app.workspace_state.roots_scanning.insert(root.clone());

    let scan_task = Task::perform(
        async move {
            let (tx, rx) = oneshot::channel();
            std::thread::spawn(move || {
                let scan_result = scan_folder_images(root.clone());
                let _ = tx.send((root, scan_result));
            });
            rx.await.expect("Thread panicked or channel closed")
        },
        Message::WorkspaceRootScanned,
    );

    handle_error_message(app, summary).chain(scan_task)
}
