use std::path::PathBuf;

use iced::Task;
use iced::futures::channel::oneshot;
use io::image_files::helpers::{FolderScanResult, scan_folder_images};

use crate::app::App;
use crate::components::sidebar_right::RightSidebarMode;
use crate::message::Message;
use crate::state::ViewMode;
use crate::state::workspace::SortingOption;
use crate::update::helpers::to_workspace_scan_result;

pub fn handle_error_message(_app: &mut App, _msg: String) -> Task<Message> {
    Task::none()
}

pub fn handle_refresh_imported_root(app: &mut App, root: PathBuf) -> Task<Message> {
    app.navigator_state.context_menu_open = false;
    app.navigator_state.context_menu_root = None;
    app.workspace_state.roots_scanning.insert(root.clone());

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
}

pub fn handle_workspace_root_scanned(
    app: &mut App,
    root: PathBuf,
    scan_result: FolderScanResult,
) -> Task<Message> {
    crate::app::startup_log(&format!(
        "WorkspaceRootScanned: {} ({} folders, {} images)",
        root.to_string_lossy(),
        scan_result.all_folders.len(),
        scan_result.all_image_paths.len()
    ));
    app.workspace_state
        .model
        .apply_root_scan_update(&root, to_workspace_scan_result(&scan_result));
    app.workspace_state.roots_scanning.remove(&root);

    if app.workspace_state.roots_scanning.is_empty() {
        crate::app::startup_log("All root scans finished");
    }

    Task::none()
}

pub fn handle_sorting_option_selected(app: &mut App, option: SortingOption) -> Task<Message> {
    println!("Selected Sorting Option {}", option);
    app.workspace_state.selected_sorting_option = option;

    // Resort the previews
    app.workspace_state.sort_previews();

    Task::none()
}

pub fn handle_sorting_direction_toggled(app: &mut App) -> Task<Message> {
    app.workspace_state.sorting_direction = app.workspace_state.sorting_direction.toggle();
    println!(
        "Sorting direction set to {:?}",
        app.workspace_state.sorting_direction
    );
    app.workspace_state.sort_previews();

    Task::none()
}

pub fn handle_preview_selected(app: &mut App, hash: String) -> Task<Message> {
    println!("Selected preview with hash {}", hash);
    app.workspace_state.selected_preview_hash = Some(hash);

    Task::none()
}

pub fn handle_view_mode_selected(app: &mut App, mode: ViewMode) -> Task<Message> {
    println!("View mode switched to {:?}", mode);

    if mode == ViewMode::Develop {
        app.right_sidebar_mode = RightSidebarMode::Operations;
        app.rebuild_pane_grid();
    } else if mode == ViewMode::Library {
        app.right_sidebar_mode = RightSidebarMode::Hidden;
        app.rebuild_pane_grid();
    }

    app.view_mode = mode;

    Task::none()
}
