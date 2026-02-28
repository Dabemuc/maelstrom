use std::path::PathBuf;

use io::catalog::ImageDO;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewMode {
    Library,
    Develop,
    NoCatalog,
}

#[derive(Debug, Clone)]
pub struct SelectionDiffData {
    pub(crate) request_id: u64,
    pub(crate) selected_path: PathBuf,
    pub(crate) images_to_add_to_catalog: Vec<PathBuf>,
    pub(crate) catalog_image_dos_to_delete: Vec<ImageDO>,
}
