use std::path::PathBuf;

use io::catalog::ImageDO;
use io::catalog::catalog_error::CatalogError;
use io::image_files::helpers::FolderScanResult;
use io::catalog::catalog::Catalog;
use previews::preview_generation::PreviewGenerationError;

use crate::components::sidebar_left::LeftSidebarMode;
use crate::components::sidebar_right::RightSidebarMode;
use crate::state::{Preview, SelectionDiffData};

#[derive(Debug, Clone)]
pub enum Message {
    LeftSidebarClicked(LeftSidebarMode),
    RightSidebarClicked(RightSidebarMode),
    CreateCatalog,
    SelectCatalog,
    CatalogLoadAttempted(Result<Catalog, CatalogError>),
    CatalogLoaded,
    NavigatorCollapseAll,
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
}
