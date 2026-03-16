use std::{collections::HashSet, path::PathBuf};

use iced::Task;
use iced::futures::channel::oneshot;
use io::{
    image_files::helpers::scan_folder_images,
    import::{create_import_plan, execute_import_plan, ImportDecision, ImportMethod},
};
use previews::preview_generation::generate_preview_for_image_with_graph;

use crate::{app::App, message::Message};
use crate::message::ImportCompletedPayload;
use crate::update::workspace::handle_error_message;

pub fn handle_import_fotos_into_managed_root(app: &mut App, root: PathBuf) -> Task<Message> {
    let Some(catalog) = app.catalog.clone() else {
        println!("Cannot add managed directory: no catalog loaded");
        return Task::none();
    };

    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        let catalog_clone = catalog.clone();
        Task::perform(
            async move {
                // Read all images to import
                let scan_result = scan_folder_images(path.clone());
                println!(
                    "Importing {} images into {:?}",
                    scan_result.all_image_paths.len(),
                    root.to_str()
                );

                let already_imported_images = match catalog_clone
                    .get_all_image_dos_for_path(root.clone())
                    .await
                {
                    Ok(value) => value,
                    Err(err) => {
                        return ImportCompletedPayload {
                            summary: format!(
                                "Failed to load already imported images for path: {}",
                                err
                            ),
                            imported_items: Vec::new(),
                            root: root.clone(),
                        };
                    }
                };
                let existing_hashes: HashSet<String> = already_imported_images
                    .into_iter()
                    .map(|img| img.hash)
                    .collect();
                let plan = create_import_plan(
                    root.clone(),
                    &scan_result,
                    &existing_hashes,
                    ImportMethod::DefaultByDate,
                );
                let imported_items = plan
                    .items
                    .iter()
                    .filter(|item| item.decision == ImportDecision::Import)
                    .cloned()
                    .collect();
                let report = execute_import_plan(plan, &catalog_clone).await;

                let summary = format!(
                    "Imported {} new images out of {} total. Skipped {}, errors {}.",
                    report.imported_count,
                    scan_result.all_image_paths.len(),
                    report.skipped_count,
                    report.errors.len()
                );

                ImportCompletedPayload {
                    summary,
                    imported_items,
                    root: root.clone(),
                }
            },
            Message::ImportCompleted,
        )
    } else {
        println!("FileDialog canceled");
        Task::none()
    }
}

pub fn handle_import_completed(
    app: &mut App,
    payload: ImportCompletedPayload,
) -> Task<Message> {
    let summary = payload.summary.clone();
    let root = payload.root.clone();
    let mut tasks: Vec<Task<Message>> = Vec::new();

    if let Some(catalog) = &app.catalog {
        let catalog_clone = catalog.clone();

        for item in payload.imported_items {
            if item.hash.is_empty() || !item.dest_path.is_file() {
                continue;
            }

            let dest_path = item.dest_path.clone();
            let hash = item.hash.clone();
            let catalog_for_task = catalog_clone.clone();

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
