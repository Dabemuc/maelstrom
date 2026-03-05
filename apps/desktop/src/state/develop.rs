use io::{
    catalog::{EditGraph, catalog::Catalog},
    image_files::supported_image_file_types::SupportedFileTypes,
};
use maelstrom_image::linear_image::LinearImage;
use std::sync::Arc;

use crate::state::{Preview, state_error::StateError};

#[derive(Debug, Clone)]
pub struct DevelopState {
    pub edit_graph: EditGraph,
    pub original_linear_image: LinearImage,
    pub developed_linear_image: Option<Arc<LinearImage>>,
}

impl DevelopState {
    pub async fn from_preview(catalog: Catalog, preview: &Preview) -> Result<Self, StateError> {
        let edit_graph = catalog.get_edit_graph(&preview.original_image.hash).await?;

        let path = preview
            .original_image
            .path
            .to_str()
            .ok_or_else(|| StateError::BoxedError("Invalid UTF-8 path".into()))?;

        let filetype = SupportedFileTypes::from_filename(path)
            .ok_or_else(|| StateError::BoxedError("Unsupported file type".into()))?;

        let linear_image = filetype.load(path, filetype.load_colorspace(path))?;

        Ok(Self {
            edit_graph,
            original_linear_image: linear_image,
            developed_linear_image: None,
        })
    }
}
