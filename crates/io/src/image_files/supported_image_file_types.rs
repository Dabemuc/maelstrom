use maelstrom_core::color::color_space::ColorSpace;
use maelstrom_image::linear_image::LinearImage;

use crate::image_files::png;

#[derive(Clone)]
pub enum SupportedFileTypes {
    PNG,
}

impl SupportedFileTypes {
    /// get file extension of enum
    pub fn _get_file_extension(&self) -> &'static str {
        match self {
            Self::PNG => "png",
        }
    }

    /// Returns the enum variant for a given filename, if supported
    pub fn from_filename(filename: &str) -> Option<Self> {
        Self::all()
            .iter()
            .cloned() // clone the enum variant
            .find(|v| v._matches(filename))
    }

    /// get all enums
    pub fn all() -> &'static [Self] {
        &[Self::PNG]
    }

    /// Checks if the input matches this type (ignoring leading dot)
    pub fn _matches(&self, input: &str) -> bool {
        let input = input.strip_prefix('.').unwrap_or(input);
        input.eq_ignore_ascii_case(self._get_file_extension())
            || input.ends_with(&format!(".{}", self._get_file_extension()))
    }

    /// Checks if input is supported by any enum variant
    pub fn is_supported(input: &str) -> bool {
        Self::all().iter().any(|v| v._matches(input))
    }

    /// call load method for enum
    pub fn load(
        &self,
        path: &str,
        space: ColorSpace,
    ) -> Result<LinearImage, Box<dyn std::error::Error>> {
        match self {
            Self::PNG => png::load_png(path, space),
        }
    }

    /// call load colorspace for enum
    pub fn load_colorspace(&self, path: &str) -> ColorSpace {
        match self {
            Self::PNG => png::load_png_colorspace(path),
        }
    }

    /// call save method for enum
    pub fn save(
        &self,
        img: &LinearImage,
        path: &str,
        space: ColorSpace,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::PNG => png::save_png(img, path, space),
        }
    }
}
