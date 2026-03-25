use iced::Task;
use services::error::ServiceError;
use services::types::{CatalogSyncResult, PreviewStatus};

use crate::app::App;
use crate::message::Message;
use crate::state::workspace::Image;
use crate::state::{Preview, PreviewState};
use crate::update::helpers::{build_quick_preview_from_image_do, refresh_selected_previews_from_cache};

pub fn handle_selection_synced(
    app: &mut App,
    result: Result<CatalogSyncResult, ServiceError>,
) -> Task<Message> {
    let data = match result {
        Ok(data) => data,
        Err(e) => {
            eprintln!("[Selection] Failed to sync selection: {}", e);
            return Task::none();
        }
    };

    if app.active_selection_request_id != Some(data.request_id) {
        return Task::none();
    }

    if app.directories_state.selected.as_ref() != Some(&data.selected_path) {
        return Task::none();
    }

    for image_do in &data.image_dos {
        app.workspace_state
            .model
            .upsert_preview_key(image_do.hash.clone(), image_do.path.clone().into());
    }

    if data.preview_data.is_empty() {
        // Fast path: build minimal previews from image_dos without reading EXIF or image
        // dimensions. Only checks whether the preview file exists — one stat() per image.
        if let Some(services) = &app.services {
            let preview_cache_dir = services.catalog.preview_cache_dir().to_path_buf();
            for image_do in &data.image_dos {
                let preview = build_quick_preview_from_image_do(&preview_cache_dir, image_do);
                app.workspace_state
                    .preview_cache
                    .insert(image_do.hash.clone(), preview);
            }
        }
    } else {
        // Full path: preview_data already contains resolved metadata (used by legacy sync path)
        for preview_data in data.preview_data {
            let preview = Preview {
                original_image: Image {
                    path: preview_data.original_image.path,
                    hash: preview_data.original_image.hash,
                    meta: preview_data.original_image.meta,
                    width: preview_data.original_image.width,
                    height: preview_data.original_image.height,
                    file_size: preview_data.original_image.file_size,
                    created_at: preview_data.original_image.created_at,
                },
                img_handle: preview_data
                    .preview_path
                    .map(iced::widget::image::Handle::from_path),
                preview_state: match preview_data.preview_status {
                    PreviewStatus::Ok => PreviewState::Ok,
                    PreviewStatus::OriginalMissing => PreviewState::OriginalMissing,
                },
            };

            app.workspace_state
                .preview_cache
                .insert(preview.original_image.hash.clone(), preview);
        }
    }

    refresh_selected_previews_from_cache(app);

    for image_do in data.catalog_image_dos_to_delete {
        if let Some(preview) = app.workspace_state.preview_cache.get_mut(&image_do.hash) {
            preview.preview_state = PreviewState::OriginalMissing;
        }

        if let Some(preview) = app.workspace_state.previews.get_mut(&image_do.hash)
            && preview.original_image.path.starts_with(&data.selected_path)
        {
            preview.preview_state = PreviewState::OriginalMissing;
        }
    }

    Task::none()
}
