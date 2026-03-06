use std::fs;
use std::path::PathBuf;

use iced::Task;
use io::catalog::ImageDO;
use previews::preview_generation::PreviewGenerationError;

use crate::app::App;
use crate::message::Message;
use crate::state::workspace::Preview;
use crate::update::helpers::build_preview_from_image_do;

pub fn handle_preview_generated(
    app: &mut App,
    result: Result<ImageDO, PreviewGenerationError>,
) -> Task<Message> {
    match result {
        Ok(image_do) => {
            if let Some(catalog) = &app.catalog {
                let mut preview = build_preview_from_image_do(catalog, &image_do);

                let preview_path = catalog.preview_cache_dir().join(format!(
                    "{}.{}",
                    image_do.hash,
                    previews::preview_generation::PREVIEW_FILE_TYPE.get_file_extension()
                ));

                if let Ok(bytes) = fs::read(&preview_path) {
                    preview.img_handle = Some(iced::widget::image::Handle::from_bytes(bytes));
                }

                app.workspace_state
                    .model
                    .upsert_preview_key(image_do.hash.clone(), PathBuf::from(&image_do.path));

                app.workspace_state
                    .preview_cache
                    .insert(image_do.hash.clone(), preview.clone());

                if let Some(selected) = app.directories_state.selected.as_ref() {
                    if preview.original_image.path.starts_with(selected) {
                        app.workspace_state
                            .previews
                            .insert(image_do.hash.clone(), preview);

                        // Add to sorted preview keys if not already present
                        if !app
                            .workspace_state
                            .sorted_preview_keys
                            .contains(&image_do.hash)
                        {
                            app.workspace_state.sorted_preview_keys.push(image_do.hash);
                        }

                        // Resort the previews
                        app.workspace_state.sort_previews();
                    }
                }
            }

            Task::none()
        }
        Err(e) => {
            println!("[Preview Generation] Failed with error: {}", e);
            Task::none()
        }
    }
}

pub fn handle_preview_data_loaded_for_image(app: &mut App, preview: Preview) -> Task<Message> {
    let hash = preview.original_image.hash.clone();

    app.workspace_state
        .model
        .upsert_preview_key(hash.clone(), preview.original_image.path.clone());

    app.workspace_state
        .preview_cache
        .insert(hash.clone(), preview.clone());

    if let Some(selected) = app.directories_state.selected.as_ref() {
        if preview.original_image.path.starts_with(selected) {
            app.workspace_state.previews.insert(hash.clone(), preview);

            // Add to sorted preview keys if not already present
            if !app.workspace_state.sorted_preview_keys.contains(&hash) {
                app.workspace_state.sorted_preview_keys.push(hash);
            }

            // Resort the previews
            app.workspace_state.sort_previews();
        }
    }

    Task::none()
}
