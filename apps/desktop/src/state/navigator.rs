use std::collections::HashSet;
use std::path::PathBuf;

pub struct NavigatorState {
    pub(crate) expanded: HashSet<PathBuf>,
    pub(crate) selected: Option<PathBuf>,
    pub(crate) context_menu_root: Option<PathBuf>,
    pub(crate) context_menu_open: bool,
}
