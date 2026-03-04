use std::path::PathBuf;

use iced::Task;
use iced::futures::channel::oneshot;
use io::image_files::helpers::scan_folder_images;

use crate::app::App;
use crate::business::cache::compare_cache_to_fs;
use crate::message::Message;
use crate::state::{PreviewState, SelectionDiffData};
use crate::update::helpers::{build_preview_from_image_do, refresh_selected_previews_from_cache};

pub fn handle_selection_catalog_loaded(
    app: &mut App,
    load_result: Result<
        (u64, PathBuf, Vec<io::catalog::ImageDO>),
        io::catalog::catalog_error::CatalogError,
    >,
) -> Task<Message> {
    match load_result {
        Ok((request_id, selected_path, image_dos)) => {
            if app.active_selection_request_id != Some(request_id) {
                return Task::none();
            }

            if app.navigator_state.selected.as_ref() != Some(&selected_path) {
                return Task::none();
            }

            for image_do in &image_dos {
                app.workspace_state
                    .model
                    .upsert_preview_key(image_do.hash.clone(), PathBuf::from(&image_do.path));
            }

            refresh_selected_previews_from_cache(app);

            let mut tasks: Vec<Task<Message>> = Vec::new();

            if let Some(catalog) = &app.catalog {
                let catalog_clone = catalog.clone();

                for image_do in &image_dos {
                    let hash = image_do.hash.clone();

                    if app.workspace_state.preview_cache.contains_key(&hash) {
                        continue;
                    }

                    let image_do = image_do.clone();
                    let catalog_for_task = catalog_clone.clone();
                    tasks.push(Task::perform(
                        async move { build_preview_from_image_do(&catalog_for_task, &image_do) },
                        Message::PreviewDataLoadedForImage,
                    ));
                }
            }

            let selected_path_for_diff = selected_path.clone();
            let image_dos_for_diff = image_dos.clone();

            tasks.push(Task::perform(
                async move {
                    let (tx, rx) = oneshot::channel();
                    std::thread::spawn(move || {
                        let selected_scan = scan_folder_images(selected_path_for_diff.clone());

                        let (images_to_add_to_catalog, catalog_image_dos_to_delete) =
                            compare_cache_to_fs(selected_scan.all_image_paths, image_dos_for_diff);

                        let _ = tx.send(SelectionDiffData {
                            request_id,
                            selected_path: selected_path_for_diff,
                            images_to_add_to_catalog,
                            catalog_image_dos_to_delete,
                        });
                    });

                    rx.await.expect("Thread panicked or channel closed")
                },
                Message::SelectionDiffComputed,
            ));

            Task::batch(tasks)
        }
        Err(e) => {
            println!("[Selection] Failed to load data for selected folder: {}", e);
            Task::none()
        }
    }
}

pub fn handle_selection_diff_computed(
    app: &mut App,
    diff_data: SelectionDiffData,
) -> Task<Message> {
    if app.active_selection_request_id != Some(diff_data.request_id) {
        return Task::none();
    }

    for image_do in diff_data.catalog_image_dos_to_delete {
        if let Some(preview) = app.workspace_state.preview_cache.get_mut(&image_do.hash) {
            preview.preview_state = PreviewState::OriginalMissing;
        }

        if let Some(preview) = app.workspace_state.previews.get_mut(&image_do.hash) {
            if preview
                .original_image
                .path
                .starts_with(&diff_data.selected_path)
            {
                preview.preview_state = PreviewState::OriginalMissing;
            }
        }
    }

    let mut add_tasks: Vec<Task<Message>> = Vec::new();
    if let Some(catalog) = &app.catalog {
        let catalog_clone = catalog.clone();

        for path in diff_data.images_to_add_to_catalog {
            let catalog_for_task = catalog_clone.clone();
            add_tasks.push(Task::perform(
                async move {
                    previews::preview_generation::generate_preview_for_image(
                        path,
                        &catalog_for_task,
                        false,
                    )
                    .await
                },
                Message::PreviewGenerated,
            ));
        }
    }

    Task::batch(add_tasks)
}
