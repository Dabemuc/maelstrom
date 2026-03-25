use std::path::PathBuf;

use iced::widget::pane_grid;
use io::catalog::ImageDO;
use io::catalog::edit_graph::{EditNodeKind, ParamValue};
use io::image_files::helpers::FolderScanResult;
use maelstrom_image::linear_image::LinearImage;
use previews::preview_generation::PreviewGenerationError;
use services::interface::Services;

use crate::components::sidebar_left::LeftSidebarMode;
use crate::components::sidebar_right::RightSidebarMode;
use crate::state::ViewMode;
use crate::state::develop::DevelopState;
use crate::state::state_error::StateError;
use crate::state::workspace::SortingOption;
use services::error::ServiceError;
use services::types::{CatalogSyncResult, ImportCompletedPayload};

#[derive(Debug, Clone)]
pub enum Message {
    LeftSidebarClicked(LeftSidebarMode),
    RightSidebarClicked(RightSidebarMode),
    PaneResized(pane_grid::ResizeEvent),
    ServicesInitialized(Result<Services, ServiceError>),
    DirectoriesCollapseAll,
    AddManagedDirectory,
    LoadManagedDirectories,
    ManagedDirectoriesLoadAttempted(Result<Vec<PathBuf>, ServiceError>),
    Notification(String),
    ToggleDirectory(PathBuf),
    SelectDirectory(PathBuf),
    OpenRootContextMenu(PathBuf),
    CloseRootContextMenu,
    RefreshManagedRoot(PathBuf),
    ImportFotos(PathBuf),
    WorkspaceRootScanned((PathBuf, FolderScanResult)),
    SelectionSynced(Result<CatalogSyncResult, ServiceError>),
    PreviewGenerated(Result<ImageDO, PreviewGenerationError>),
    ImportCompleted(ImportCompletedPayload),
    SortingOptionSelected(SortingOption),
    SortingDirectionToggled,
    PreviewDoubleClicked(String),
    ViewModeSelected(ViewMode),
    PreviewSelected(String),
    DevelopStateLoaded(Result<DevelopState, StateError>),
    ImageDeveloped(LinearImage),
    DevelopZoomSet(f32),
    DevelopZoomBy(f32),
    DevelopZoomSetPan {
        zoom: f32,
        pan: [f32; 2],
    },
    DevelopFitToScreen,
    DevelopPanBy {
        delta: [f32; 2],
    },
    DevelopParamChanged {
        kind: EditNodeKind,
        name: String,
        value: ParamValue,
    },
    DevelopParamInputChanged {
        kind: EditNodeKind,
        name: String,
        value: String,
    },
    DevelopSaveRequested,
    DevelopSaveCompleted(Result<(), ServiceError>),
    DevelopExportRequested,
}
