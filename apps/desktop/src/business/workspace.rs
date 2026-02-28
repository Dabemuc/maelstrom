use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub type PreviewKey = String;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderNode {
    pub path: PathBuf,
    pub direct_image_count: usize,
    pub total_image_count: usize,
    pub children: Vec<PathBuf>,
}

impl FolderNode {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            direct_image_count: 0,
            total_image_count: 0,
            children: Vec::new(),
        }
    }
}

/// Input payload produced by a filesystem scan.
///
/// This is intentionally UI-agnostic and can be reused for:
/// - navigator counts/tree generation
/// - cache refresh diffing
#[derive(Debug, Clone, Default)]
pub struct WorkspaceScanResult {
    /// All image paths discovered during the scan.
    pub all_image_paths: Vec<PathBuf>,
    /// All folders discovered during the scan (including roots and empty folders).
    pub all_folders: HashSet<PathBuf>,
    /// Direct-image counts only (images whose parent is this folder).
    pub direct_counts_by_folder: HashMap<PathBuf, usize>,
}

impl WorkspaceScanResult {
    pub fn new(
        all_image_paths: Vec<PathBuf>,
        all_folders: HashSet<PathBuf>,
        direct_counts_by_folder: HashMap<PathBuf, usize>,
    ) -> Self {
        Self {
            all_image_paths,
            all_folders,
            direct_counts_by_folder,
        }
    }

    /// Helper for incremental migration:
    /// if only image paths are available, derive a minimal scan result.
    ///
    /// Note: this cannot discover truly empty folders because no directory walk
    /// data is available.
    pub fn from_image_paths(imported_roots: &[PathBuf], image_paths: Vec<PathBuf>) -> Self {
        let mut all_folders: HashSet<PathBuf> = imported_roots.iter().cloned().collect();
        let mut direct_counts_by_folder: HashMap<PathBuf, usize> = HashMap::new();

        for image in &image_paths {
            if let Some(parent) = image.parent() {
                let parent_buf = parent.to_path_buf();
                all_folders.insert(parent_buf.clone());
                *direct_counts_by_folder.entry(parent_buf).or_insert(0) += 1;
            }

            for root in imported_roots {
                if image.starts_with(root) {
                    for ancestor in image.ancestors() {
                        let ancestor_buf = ancestor.to_path_buf();
                        if ancestor_buf == *root {
                            all_folders.insert(ancestor_buf);
                            break;
                        }

                        if ancestor_buf.starts_with(root) {
                            all_folders.insert(ancestor_buf);
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        Self {
            all_image_paths: image_paths,
            all_folders,
            direct_counts_by_folder,
        }
    }
}

/// Persistent workspace model that unifies:
/// - folder tree/count state
/// - preview cache index state
#[derive(Debug, Clone, Default)]
pub struct WorkspaceModel {
    pub root_folders: Vec<PathBuf>,
    pub folder_index: HashMap<PathBuf, FolderNode>,

    /// Stable list of all scanned images for cache refresh diffing.
    pub all_image_paths: Vec<PathBuf>,

    /// Global preview cache (stable key -> original path).
    /// Actual preview payload stays in the existing app state for incremental migration.
    pub preview_cache: HashMap<PreviewKey, PathBuf>,

    /// Folder -> ordered preview keys used by UI rendering.
    pub previews_by_folder: HashMap<PathBuf, Vec<PreviewKey>>,
}

impl WorkspaceModel {
    pub fn clear(&mut self) {
        self.root_folders.clear();
        self.folder_index.clear();
        self.all_image_paths.clear();
        self.preview_cache.clear();
        self.previews_by_folder.clear();
    }

    /// Replaces folder/count model from a fresh scan in one pass.
    pub fn apply_scan(&mut self, imported_roots: &[PathBuf], scan: WorkspaceScanResult) {
        self.root_folders = imported_roots.to_vec();
        self.all_image_paths = scan.all_image_paths;
        self.folder_index = build_folder_index(
            imported_roots,
            scan.all_folders,
            scan.direct_counts_by_folder,
        );
    }

    /// Incrementally updates a single imported root with a fresh scan result.
    ///
    /// This keeps data for other roots untouched and only replaces:
    /// - folder/count state under `root`
    /// - scanned image list entries under `root`
    pub fn apply_root_scan_update(&mut self, root: &Path, scan: WorkspaceScanResult) {
        let root_buf = root.to_path_buf();

        if !self.root_folders.iter().any(|p| p == &root_buf) {
            self.root_folders.push(root_buf.clone());
            self.root_folders.sort();
            self.root_folders.dedup();
        }

        // Merge all image paths: keep non-root entries, replace root subtree entries.
        self.all_image_paths.retain(|p| !is_within_root(p, root));
        self.all_image_paths.extend(scan.all_image_paths.clone());

        // Merge folder + direct-count data from existing index with new root scan.
        let mut merged_folders: HashSet<PathBuf> = self
            .folder_index
            .keys()
            .filter(|p| !is_within_root(p, root))
            .cloned()
            .collect();

        let mut merged_direct_counts: HashMap<PathBuf, usize> = self
            .folder_index
            .iter()
            .filter(|(path, _)| !is_within_root(path, root))
            .map(|(path, node)| (path.clone(), node.direct_image_count))
            .collect();

        merged_folders.extend(scan.all_folders.clone());
        merged_direct_counts.extend(scan.direct_counts_by_folder.clone());

        self.folder_index =
            build_folder_index(&self.root_folders, merged_folders, merged_direct_counts);

        // Keep preview indexes stable, but drop folder mappings that no longer exist.
        self.previews_by_folder
            .retain(|folder, _| self.folder_index.contains_key(folder));
    }

    pub fn total_count_for(&self, folder: &Path) -> usize {
        self.folder_index
            .get(folder)
            .map(|n| n.total_image_count)
            .unwrap_or(0)
    }

    pub fn direct_count_for(&self, folder: &Path) -> usize {
        self.folder_index
            .get(folder)
            .map(|n| n.direct_image_count)
            .unwrap_or(0)
    }

    pub fn children_for(&self, folder: &Path) -> Vec<PathBuf> {
        self.folder_index
            .get(folder)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    pub fn set_previews_by_folder<I>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (PathBuf, PreviewKey)>,
    {
        self.previews_by_folder.clear();
        for (folder, key) in entries {
            self.previews_by_folder.entry(folder).or_default().push(key);
        }

        for keys in self.previews_by_folder.values_mut() {
            keys.sort();
            keys.dedup();
        }
    }

    pub fn upsert_preview_key(&mut self, key: PreviewKey, original_path: PathBuf) {
        self.preview_cache
            .insert(key.clone(), original_path.clone());

        if let Some(mut ancestor) = original_path.parent() {
            loop {
                let folder = ancestor.to_path_buf();

                if self.folder_index.contains_key(&folder) {
                    let keys = self.previews_by_folder.entry(folder).or_default();
                    if !keys.iter().any(|k| k == &key) {
                        keys.push(key.clone());
                    }
                }

                let Some(next) = ancestor.parent() else {
                    break;
                };
                ancestor = next;
            }
        }
    }

    pub fn remove_preview_key(&mut self, key: &str) {
        self.preview_cache.remove(key);

        for keys in self.previews_by_folder.values_mut() {
            keys.retain(|k| k != key);
        }

        self.previews_by_folder.retain(|_, keys| !keys.is_empty());
    }

    /// Returns cached preview keys for the selected folder.
    /// This is designed to avoid clearing global previews when changing selection.
    pub fn preview_keys_for_selected_folder(&self, folder: &Path) -> Vec<PreviewKey> {
        self.previews_by_folder
            .get(folder)
            .cloned()
            .unwrap_or_default()
    }
}

fn build_folder_index(
    imported_roots: &[PathBuf],
    discovered_folders: HashSet<PathBuf>,
    direct_counts_by_folder: HashMap<PathBuf, usize>,
) -> HashMap<PathBuf, FolderNode> {
    let mut folders = discovered_folders;
    folders.extend(imported_roots.iter().cloned());

    let mut index: HashMap<PathBuf, FolderNode> = HashMap::new();

    for folder in &folders {
        index
            .entry(folder.clone())
            .or_insert_with(|| FolderNode::new(folder.clone()));
    }

    // Parent/child links (only for folders inside imported roots and discovered set).
    for folder in &folders {
        if let Some(parent) = folder.parent() {
            let parent_buf = parent.to_path_buf();

            if folders.contains(&parent_buf) {
                let parent_node = index
                    .entry(parent_buf.clone())
                    .or_insert_with(|| FolderNode::new(parent_buf.clone()));

                if !parent_node.children.iter().any(|p| p == folder) {
                    parent_node.children.push(folder.clone());
                }
            }
        }
    }

    // Direct counts and initial totals.
    for (folder, direct) in direct_counts_by_folder {
        let node = index
            .entry(folder.clone())
            .or_insert_with(|| FolderNode::new(folder));
        node.direct_image_count = direct;
        node.total_image_count = direct;
    }

    // Ensure all nodes have total initialized to direct.
    for node in index.values_mut() {
        if node.total_image_count == 0 {
            node.total_image_count = node.direct_image_count;
        }
        node.children.sort();
    }

    // Roll up totals from deepest folders to roots.
    let mut paths_by_depth: Vec<PathBuf> = index.keys().cloned().collect();
    paths_by_depth.sort_by_key(|p| std::cmp::Reverse(path_depth(p)));

    for folder in paths_by_depth {
        let total = index
            .get(&folder)
            .map(|n| n.total_image_count)
            .unwrap_or_default();

        if let Some(parent) = folder.parent() {
            let parent_buf = parent.to_path_buf();
            if let Some(parent_node) = index.get_mut(&parent_buf) {
                parent_node.total_image_count += total;
            }
        }
    }

    index
}

fn is_within_root(path: &Path, root: &Path) -> bool {
    path == root || path.starts_with(root)
}

fn path_depth(path: &Path) -> usize {
    path.components().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_direct_and_total_counts() {
        let root = PathBuf::from("/photos");
        let child = PathBuf::from("/photos/2024");
        let grandchild = PathBuf::from("/photos/2024/trip");

        let all_folders = HashSet::from([root.clone(), child.clone(), grandchild.clone()]);
        let direct_counts = HashMap::from([(root.clone(), 1), (child.clone(), 2), (grandchild, 3)]);

        let mut model = WorkspaceModel::default();
        model.apply_scan(
            std::slice::from_ref(&root),
            WorkspaceScanResult::new(Vec::new(), all_folders, direct_counts),
        );

        assert_eq!(model.direct_count_for(&root), 1);
        assert_eq!(model.total_count_for(&root), 6);
        assert_eq!(model.total_count_for(&child), 5);
    }

    #[test]
    fn keeps_preview_index_per_folder() {
        let mut model = WorkspaceModel::default();

        model.upsert_preview_key("h1".to_string(), PathBuf::from("/photos/a.jpg"));
        model.upsert_preview_key("h2".to_string(), PathBuf::from("/photos/b.jpg"));

        let keys = model.preview_keys_for_selected_folder(Path::new("/photos"));
        assert_eq!(keys.len(), 2);

        model.remove_preview_key("h1");
        let keys_after = model.preview_keys_for_selected_folder(Path::new("/photos"));
        assert_eq!(keys_after, vec!["h2".to_string()]);
    }
}
