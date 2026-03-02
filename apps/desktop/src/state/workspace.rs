use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use iced::widget::image::Handle;

use crate::business::workspace::WorkspaceModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    pub path_to_original: PathBuf,
    pub hash: String,
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
        self.path_to_original.hash(state);
        self.hash.hash(state);
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

    // Current render set for selected folder (kept for compatibility with existing center stage).
    pub previews: HashMap<String, Preview>,
    // pub sorted_preview_keys: Vec<String>,

    pub handle_to_missing_preview_placeholder: Handle,

    // State to hold sorting options as well as currently selected
    pub selected_sorting_option: SortingOption,
}
