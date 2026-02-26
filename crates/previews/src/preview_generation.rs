use std::path::PathBuf;

use graph::{graph::Graph, node::Backend};
use io::{
    catalog::{
        ImageDO,
        catalog::{CACHE_DIR_NAME, Catalog},
        catalog_error::CatalogError,
    },
    image_files::supported_image_file_types::{SaveOptions, SupportedFileTypes},
};
use maelstrom_core::{color::color_space::ColorSpace, hash::hash_file};
use ops::downsample_fixed::DownsampleFixed;
use thiserror::Error;

pub const PREVIEW_FILE_TYPE: SupportedFileTypes = SupportedFileTypes::JPEG;

#[derive(Debug, Error, Clone)]
pub enum PreviewGenerationError {
    #[error("CatalogError: {0:?}")]
    CatalogError(CatalogError),

    #[error("Filetype not supported")]
    FiletypeNotSupported,

    #[error("Image entry already exists in catalog")]
    AlreadyExists,

    #[error("Failed to generate hash for image")]
    HashingError,

    #[error("Skipped")]
    Skipped,

    #[error("Error when saving to fs: {0:?}")]
    SavingError(String),
}

pub async fn generate_preview_for_image(
    path_to_img: PathBuf,
    catalog: &Catalog,
    overwrite_if_exists: bool,
) -> Result<ImageDO, PreviewGenerationError> {
    // 1. get file name
    let filename = path_to_filename(path_to_img.clone());

    // 2. check if image supported
    if !SupportedFileTypes::is_supported(&filename) {
        println!(
            "Preview generation failed. Filetype is not supported: {:?}",
            path_to_img
        );
        return Err(PreviewGenerationError::FiletypeNotSupported);
    }

    let image_file_type = SupportedFileTypes::from_filename(&filename).unwrap();

    // 3. Hash image content & file name
    let content_hash = match hash_file(&path_to_img) {
        Ok(h) => h,
        Err(e) => {
            eprintln!(
                "Preview generation failed. Error hashing image {:?}: {}",
                path_to_img, e
            );
            return Err(PreviewGenerationError::HashingError);
        }
    };

    // 4. Check catalog
    let exists = match catalog.image_exists(&content_hash).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Preview generation failed. Catalog check failed: {}", e);
            return Err(PreviewGenerationError::AlreadyExists);
        }
    };

    if exists && !overwrite_if_exists {
        println!(
            "Preview generation skipped. Preview already exists for {:?}",
            path_to_img
        );
        return Err(PreviewGenerationError::Skipped);
    }

    // 5. Load image
    let image_linear = image_file_type
        .load(
            path_to_img.to_str().unwrap(),
            image_file_type.load_colorspace(path_to_img.to_str().unwrap()),
        )
        .unwrap();

    // 6. Define preview generation graph
    let mut graph = Graph::new();
    graph.add_node(DownsampleFixed {
        max_width: 480,
        max_height: 480,
    });

    // 7. Generate preview
    let result = graph.execute(image_linear, Backend::Cpu);
    println!(
        "Generated preview for {:?} with resolution: {}x{}",
        path_to_img, result.width, result.height
    );

    // 8. Save preview using hash as filename
    let preview_path_buf = catalog.root().join(CACHE_DIR_NAME).join(format!(
        "{}.{}",
        content_hash,
        PREVIEW_FILE_TYPE.get_file_extension()
    ));
    let preview_path = preview_path_buf.to_str().unwrap();
    if let Err(e) = PREVIEW_FILE_TYPE.save(
        &result,
        &preview_path,
        ColorSpace::Srgb,
        Some(SaveOptions { quality: 50 }),
    ) {
        eprintln!("Error saving preview for {:?}: {}", path_to_img, e);
        return Err(PreviewGenerationError::SavingError(e.to_string()));
    }

    // 9. Insert image into DB (if new)
    if !exists {
        match catalog
            .add_image(&content_hash, path_to_img.to_str().unwrap())
            .await
        {
            Ok(image_do) => return Ok(image_do),
            Err(e) => {
                eprintln!("Failed to insert image hash into catalog: {}", e);
                return Err(PreviewGenerationError::CatalogError(e));
            }
        }
    } else {
        return Err(PreviewGenerationError::AlreadyExists);
    };
}

fn path_to_filename(path: PathBuf) -> String {
    match path.file_name() {
        Some(os_str) => match os_str.to_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!("Failed to convert file name to UTF-8: {:?}", path);
                String::new()
            }
        },
        None => {
            eprintln!("Path has no file name: {:?}", path);
            String::new()
        }
    }
}
