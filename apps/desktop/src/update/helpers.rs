use std::path::PathBuf;

use iced::widget::image::Handle;
use io::catalog::ImageDO;
use io::catalog::catalog::Catalog;
use io::image_files::helpers::FolderScanResult;
use previews::preview_generation::PREVIEW_FILE_TYPE;

use crate::app::App;
use crate::business::workspace::WorkspaceScanResult;
use crate::state::{Preview, PreviewState};

/// Rebuilds `workspace_state.previews` from the persistent preview cache
/// for the currently selected folder.
pub fn refresh_selected_previews_from_cache(app: &mut App) {
    app.workspace_state.previews.clear();
    app.workspace_state.sorted_preview_keys.clear();

    let Some(selected) = app.navigator_state.selected.as_ref() else {
        return;
    };

    for key in app
        .workspace_state
        .model
        .preview_keys_for_selected_folder(selected)
    {
        if let Some(preview) = app.workspace_state.preview_cache.get(&key) {
            app.workspace_state
                .previews
                .insert(key.clone(), preview.clone());
            app.workspace_state.sorted_preview_keys.push(key.clone());
        }
    }

    // Sort the previews based on the current sorting option
    app.workspace_state.sort_previews();
}

/// Converts a `FolderScanResult` (from the `io` crate) into a `WorkspaceScanResult`
/// (the business-layer type used by `WorkspaceModel`).
pub fn to_workspace_scan_result(scan_result: &FolderScanResult) -> WorkspaceScanResult {
    WorkspaceScanResult::new(
        scan_result.all_image_paths.clone(),
        scan_result.all_folders.clone(),
        scan_result.direct_image_counts.clone(),
    )
}

/// Builds a `Preview` from a catalog `ImageDO`, resolving the cached preview
/// file path and determining whether the original is present.
pub fn build_preview_from_image_do(catalog: &Catalog, image_do: &ImageDO) -> Preview {
    let path = catalog.root().join(catalog.cache_dir()).join(format!(
        "{}.{}",
        image_do.hash,
        PREVIEW_FILE_TYPE.get_file_extension()
    ));

    Preview {
        path_to_original: PathBuf::from(&image_do.path),
        hash: image_do.hash.clone(),
        img_handle: if path.exists() {
            Some(Handle::from_path(path.clone()))
        } else {
            None
        },
        preview_state: if path.exists() {
            PreviewState::Ok
        } else {
            PreviewState::OriginalMissing
        },
    }
}
