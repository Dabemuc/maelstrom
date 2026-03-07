use std::path::PathBuf;

use iced::widget::pane_grid;
use io::catalog::ImageDO;
use io::catalog::catalog::Catalog;
use io::catalog::catalog_error::CatalogError;
use io::catalog::edit_graph::{EditNodeKind, ParamValue};
use io::image_files::helpers::FolderScanResult;
use maelstrom_image::linear_image::LinearImage;
use previews::preview_generation::PreviewGenerationError;

use crate::components::sidebar_left::LeftSidebarMode;
use crate::components::sidebar_right::RightSidebarMode;
use crate::state::ViewMode;
use crate::state::develop::DevelopState;
use crate::state::state_error::StateError;
use crate::state::workspace::SortingOption;
use crate::state::{Preview, SelectionDiffData};

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
    ImportDirectory,
    LoadImportedDirectories,
    ImportedDirectoriesLoadAttempted(Result<Vec<PathBuf>, CatalogError>),
    ErrorMessage(String),
    ToggleDirectory(PathBuf),
    SelectDirectory(PathBuf),
    OpenRootContextMenu(PathBuf),
    CloseRootContextMenu,
    RefreshImportedRoot(PathBuf),
    WorkspaceRootScanned((PathBuf, FolderScanResult)),
    SelectionCatalogLoaded(Result<(u64, PathBuf, Vec<ImageDO>), CatalogError>),
    SelectionDiffComputed(SelectionDiffData),
    PreviewDataLoadedForImage(Preview),
    PreviewGenerated(Result<ImageDO, PreviewGenerationError>),
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
