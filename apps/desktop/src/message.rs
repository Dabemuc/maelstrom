use std::path::PathBuf;

use iced::widget::pane_grid;
use io::catalog::catalog::Catalog;
use io::catalog::catalog_error::CatalogError;
use io::catalog::edit_graph::{EditNodeKind, ParamValue};
use io::catalog::ImageDO;
use io::image_files::helpers::FolderScanResult;
use io::import::ImportItem;
use maelstrom_image::linear_image::LinearImage;
use previews::preview_generation::PreviewGenerationError;

use crate::components::sidebar_left::LeftSidebarMode;
use crate::components::sidebar_right::RightSidebarMode;
use crate::state::develop::DevelopState;
use crate::state::state_error::StateError;
use crate::state::workspace::SortingOption;
use crate::state::ViewMode;
use services::error::ServiceError;
use services::types::SelectionSyncResult;

#[derive(Debug, Clone)]
pub struct ImportCompletedPayload {
    pub summary: String,
    pub imported_items: Vec<ImportItem>,
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub enum Message {
    LeftSidebarClicked(LeftSidebarMode),
    RightSidebarClicked(RightSidebarMode),
    PaneResized(pane_grid::ResizeEvent),
    CreateCatalog,
    SelectCatalog,
    CatalogLoadAttempted(Result<Catalog, CatalogError>),
    CatalogLoaded,
    DirectoriesCollapseAll,
    AddManagedDirectory,
    LoadManagedDirectories,
    ManagedDirectoriesLoadAttempted(Result<Vec<PathBuf>, CatalogError>),
    Notification(String),
    ToggleDirectory(PathBuf),
    SelectDirectory(PathBuf),
    OpenRootContextMenu(PathBuf),
    CloseRootContextMenu,
    RefreshManagedRoot(PathBuf),
    ImportFotos(PathBuf),
    WorkspaceRootScanned((PathBuf, FolderScanResult)),
    SelectionSynced(Result<SelectionSyncResult, ServiceError>),
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
    DevelopSaveCompleted(Result<(), CatalogError>),
    DevelopExportRequested,
}
