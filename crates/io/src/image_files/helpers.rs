use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::image_files::supported_image_file_types::SupportedFileTypes;

/// Result of a one-pass recursive filesystem scan rooted at a folder.
///
/// This captures:
/// - every discovered image path
/// - every discovered folder (including empty folders)
/// - direct image counts per folder (images directly in that folder, not descendants)
#[derive(Debug, Clone, Default)]
pub struct FolderScanResult {
    pub root: PathBuf,
    pub all_image_paths: Vec<PathBuf>,
    pub all_folders: HashSet<PathBuf>,
    pub direct_image_counts: HashMap<PathBuf, usize>,
}

impl FolderScanResult {
    pub fn new(root: PathBuf) -> Self {
        let mut result = Self {
            root: root.clone(),
            all_image_paths: Vec::new(),
            all_folders: HashSet::new(),
            direct_image_counts: HashMap::new(),
        };

        result.all_folders.insert(root);
        result
    }
}

/// One-pass recursive scan of a folder tree.
///
/// Traverses each directory entry once and returns a structured result suitable
/// for both:
/// - image-path consumers
/// - folder/count model construction
pub fn scan_folder_images(root: impl AsRef<Path>) -> FolderScanResult {
    fn walk(path: &Path, result: &mut FolderScanResult) {
        result.all_folders.insert(path.to_path_buf());

        let Ok(entries) = fs::read_dir(path) else {
            return;
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                walk(&entry_path, result);
                continue;
            }

            if !entry_path.is_file() {
                continue;
            }

            if let Some(ext) = entry_path.extension().and_then(|e| e.to_str())
                && SupportedFileTypes::is_supported(ext.to_ascii_lowercase().as_str())
            {
                result.all_image_paths.push(entry_path);
                *result
                    .direct_image_counts
                    .entry(path.to_path_buf())
                    .or_insert(0) += 1;
            }
        }
    }

    let root = root.as_ref().to_path_buf();
    let mut result = FolderScanResult::new(root.clone());

    if root.is_dir() {
        walk(&root, &mut result);
    }

    result
}

/// Legacy helper retained for compatibility.
///
/// Uses the new one-pass scan implementation and returns only image paths.
pub fn collect_images_in_folder(path: PathBuf) -> Vec<PathBuf> {
    scan_folder_images(path).all_image_paths
}
