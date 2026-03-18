use std::path::PathBuf;

use io::catalog::ImageDO;
use io::metadata::metadata::Metadata;
use previews::preview_generation::PreviewGenerationError;

#[derive(Debug, Clone)]
pub struct ImageData {
    pub path: PathBuf,
    pub hash: String,
    pub meta: Option<Metadata>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size: Option<u64>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewStatus {
    Ok,
    OriginalMissing,
}

#[derive(Debug, Clone)]
pub struct PreviewData {
    pub original_image: ImageData,
    pub preview_path: Option<PathBuf>,
    pub preview_status: PreviewStatus,
}

#[derive(Debug, Clone)]
pub struct SelectionSyncResult {
    pub request_id: u64,
    pub selected_path: PathBuf,
    pub image_dos: Vec<ImageDO>,
    pub preview_data: Vec<PreviewData>,
    pub images_to_add_to_catalog: Vec<PathBuf>,
    pub catalog_image_dos_to_delete: Vec<ImageDO>,
    pub generated: Vec<Result<ImageDO, PreviewGenerationError>>,
}
