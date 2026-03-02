use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::Instant;

use iced::widget::image::Handle;

use crate::business::workspace::WorkspaceModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    pub original_image: Image,
    pub img_handle: Option<Handle>,
    pub preview_state: PreviewState,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PreviewState {
    Ok,
    OriginalMissing,
}

impl Hash for Preview {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.original_image.path.hash(state);
        self.original_image.hash.hash(state);
        self.preview_state.hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortingOption {
    FileName,
}

impl std::fmt::Display for SortingOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::FileName => "Name",
        })
    }
}

pub struct WorkspaceState {
    pub model: WorkspaceModel,

    // Imported roots currently being scanned (initial load + reselection refreshes).
    pub roots_scanning: HashSet<PathBuf>,

    // Persistent preview payload cache (hash -> Preview).
    pub preview_cache: HashMap<String, Preview>,

    // Current render set for selected folder.
    pub previews: HashMap<String, Preview>,

    // Sorted preview keys for display ordering
    pub sorted_preview_keys: Vec<String>,

    pub handle_to_missing_preview_placeholder: Handle,

    // State to hold sorting options as well as currently selected
    pub selected_sorting_option: SortingOption,
}

impl WorkspaceState {
    /// Sort previews according to the selected sorting option
    pub fn sort_previews(&mut self) {
        println!(
            "[Workspace State] Sorting previews by {}",
            self.selected_sorting_option
        );
        let time_before_sort = Instant::now();
        let cmp = match self.selected_sorting_option {
            SortingOption::FileName => |a: &String, b: &String| {
                let name_a = self
                    .previews
                    .get(a)
                    .map(|p| p.original_image.path.file_name().unwrap_or_default())
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();
                let name_b = self
                    .previews
                    .get(b)
                    .map(|p| p.original_image.path.file_name().unwrap_or_default())
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();
                name_a.cmp(&name_b)
            },
        };

        self.sorted_preview_keys.sort_by(cmp);
        println!(
            "[Workspace State] Sorting took {}ms",
            time_before_sort.elapsed().as_millis()
        );
    }
}
