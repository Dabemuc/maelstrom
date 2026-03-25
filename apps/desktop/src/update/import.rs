use std::path::PathBuf;

use iced::Task;
use iced::futures::channel::oneshot;
use io::image_files::helpers::scan_folder_images;
use previews::preview_generation::generate_preview_for_image_with_graph;
use services::types::ImportCompletedPayload;

use crate::update::workspace::handle_error_message;
use crate::{app::App, message::Message};

pub fn handle_import_fotos_into_managed_root(app: &mut App, root: PathBuf) -> Task<Message> {
    let Some(services) = app.services.clone() else {
        println!("Cannot add managed directory: no services loaded");
        return Task::none();
    };

    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        Task::perform(
            async move {
                services
                    .catalog
                    .import_fotos_into_managed_dir_with_strategy(
                        services::ImportStrategy::DefaultByDate,
                        path,
                        root,
                    )
                    .await
            },
            Message::ImportCompleted,
        )
    } else {
        println!("FileDialog canceled");
        Task::none()
    }
}

pub fn handle_import_completed(app: &mut App, payload: ImportCompletedPayload) -> Task<Message> {
    let summary = payload.summary.clone();
    let root = payload.root.clone();
    let mut tasks: Vec<Task<Message>> = Vec::new();

    if let Some(services) = &app.services {
        for item in payload.imported_items {
            if item.hash.is_empty() || !item.dest_path.is_file() {
                continue;
            }

            let dest_path = item.dest_path.clone();
            let hash = item.hash.clone();
            let catalog_for_task = services.catalog.get_catalog_ref().clone();

            tasks.push(Task::perform(
                async move {
                    generate_preview_for_image_with_graph(
                        dest_path,
                        hash,
                        io::catalog::EditGraph::default(),
                        &catalog_for_task,
                    )
                    .await
                },
                Message::PreviewGenerated,
            ));
        }
    }

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

    handle_error_message(app, summary)
        .chain(Task::batch(tasks))
        .chain(scan_task)
}
