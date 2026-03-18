use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use image::image_dimensions;
use io::catalog::{catalog::Catalog, ImageDO};
use io::metadata::metadata::Metadata;
use previews::preview_generation::PREVIEW_FILE_TYPE;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::types::{ImageData, PreviewData, PreviewStatus};

pub fn preview_data_from_image_do(catalog: &Catalog, image_do: &ImageDO) -> PreviewData {
    let preview_path = catalog.preview_cache_dir().join(format!(
        "{}.{}",
        image_do.hash,
        PREVIEW_FILE_TYPE.get_file_extension()
    ));

    let image_path = PathBuf::from(&image_do.path);
    let meta = Metadata::read_exif(&image_path).ok();

    let (width, height) = image_dimensions(&image_path)
        .map(|(w, h)| (Some(w), Some(h)))
        .unwrap_or((None, None));

    let file_size = fs::metadata(&image_path).map(|meta| meta.len()).ok();
    let created_at = fs::metadata(&image_path)
        .ok()
        .and_then(|meta| meta.created().ok())
        .and_then(format_system_time);

    let preview_exists = preview_path.exists();

    PreviewData {
        original_image: ImageData {
            path: image_path,
            hash: image_do.hash.clone(),
            meta,
            width,
            height,
            file_size,
            created_at,
        },
        preview_path: if preview_exists {
            Some(preview_path)
        } else {
            None
        },
        preview_status: if preview_exists {
            PreviewStatus::Ok
        } else {
            PreviewStatus::OriginalMissing
        },
    }
}

fn format_system_time(time: SystemTime) -> Option<String> {
    OffsetDateTime::from(time).format(&Rfc3339).ok()
}
