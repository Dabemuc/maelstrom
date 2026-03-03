use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

use iced::widget::image::Handle;
use iced::widget::{column, pane_grid};
use iced::{Element, Length, Task};

use io::catalog::catalog::{CATALOG_FILE_NAME, CATALOG_FOLDER_NAME, Catalog};

use crate::business::workspace::WorkspaceModel;
use crate::components::center_stage::center_stage;
use crate::components::control_panel_bottom::control_panel_bottom;
use crate::components::control_panel_top::control_panel_top;
use crate::components::divider::divider;
use crate::components::sidebar_left::{LeftSidebarMode, sidebar_left};
use crate::components::sidebar_right::{RightSidebarMode, sidebar_right};
use crate::message::Message;
use crate::state::workspace::SortingOption;
use crate::state::{NavigatorState, ViewMode, WorkspaceState};
use crate::{theme, update};

pub struct App {
    pub left_sidebar_mode: LeftSidebarMode,
    pub right_sidebar_mode: RightSidebarMode,
    pub view_mode: ViewMode,
    pub catalog: Option<Catalog>,
    pub imported_dirs: Vec<PathBuf>,
    pub navigator_state: NavigatorState,
    pub workspace_state: WorkspaceState,
    pub(crate) selection_request_seq: u64,
    pub(crate) active_selection_request_id: Option<u64>,
    pub(crate) pane_grid_state: pane_grid::State<PaneKind>,
    pub(crate) left_split: Option<pane_grid::Split>,
    pub(crate) right_split: Option<pane_grid::Split>,
    pub(crate) left_ratio: f32,
    pub(crate) right_ratio: f32,
}

static APP_START: OnceLock<Instant> = OnceLock::new();

pub fn startup_elapsed_ms() -> u128 {
    APP_START.get_or_init(Instant::now).elapsed().as_millis()
}

pub fn startup_log(event: &str) {
    println!("[Startup +{}ms] {}", startup_elapsed_ms(), event);
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        APP_START.get_or_init(Instant::now);
        startup_log("App::new started");

        let left_ratio = default_left_ratio();
        let right_ratio = default_right_ratio();
        let layout = build_pane_grid_layout(
            LeftSidebarMode::Navigator,
            RightSidebarMode::Hidden,
            left_ratio,
            right_ratio,
        );

        let app = Self {
            left_sidebar_mode: LeftSidebarMode::Navigator,
            right_sidebar_mode: RightSidebarMode::Hidden,
            view_mode: ViewMode::NoCatalog,
            catalog: None,
            imported_dirs: Vec::new(),
            navigator_state: NavigatorState {
                expanded: HashSet::new(),
                selected: None,
                context_menu_root: None,
                context_menu_open: false,
            },
            workspace_state: WorkspaceState {
                model: WorkspaceModel::default(),
                roots_scanning: HashSet::new(),
                preview_cache: HashMap::new(),
                previews: HashMap::new(),
                sorted_preview_keys: Vec::new(),
                handle_to_missing_preview_placeholder: Handle::from_bytes(
                    include_bytes!("../assets/static/image_missing.png").to_vec(),
                ),
                selected_sorting_option: SortingOption::FileName,
                selected_preview_hash: None,
            },
            selection_request_seq: 0,
            active_selection_request_id: None,
            pane_grid_state: layout.state,
            left_split: layout.left_split,
            right_split: layout.right_split,
            left_ratio,
            right_ratio,
        };

        let config_base = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("maelstrom");

        if !config_base.exists() {
            println!("User config dir doesnt exists at {:?}", config_base);
            startup_log("Config base missing; startup idle");
            return (app, Task::none());
        }

        let catalog_root = config_base.join(CATALOG_FOLDER_NAME);

        let startup_task = if catalog_root.join(CATALOG_FILE_NAME).exists() {
            startup_log("Catalog file exists; loading catalog");
            Task::perform(
                Catalog::load(catalog_root.clone()),
                Message::CatalogLoadAttempted,
            )
        } else {
            println!("default catalog not found, creating at: {:?}", catalog_root);
            startup_log("Catalog missing; creating default catalog");
            Task::perform(
                Catalog::create(config_base.clone()),
                Message::CatalogLoadAttempted,
            )
        };

        startup_log("App::new finished, startup task dispatched");
        (app, startup_task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        update::update(self, message)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let main_content = pane_grid(&self.pane_grid_state, |_, pane, _| {
            pane_grid::Content::new(match pane {
                PaneKind::LeftSidebar => sidebar_left(self),
                PaneKind::CenterStage => center_stage(self),
                PaneKind::RightSidebar => sidebar_right(self),
            })
        })
        .on_resize(8, Message::PaneResized)
        .min_size(200)
        .height(Length::Fill);

        column![
            control_panel_top(self),
            divider(false),
            main_content,
            control_panel_bottom(self),
        ]
        .into()
    }

    pub fn theme(&self) -> iced::Theme {
        theme::maelstrom_theme()
    }

    pub fn rebuild_pane_grid(&mut self) {
        let layout = build_pane_grid_layout(
            self.left_sidebar_mode,
            self.right_sidebar_mode,
            self.left_ratio,
            self.right_ratio,
        );
        self.pane_grid_state = layout.state;
        self.left_split = layout.left_split;
        self.right_split = layout.right_split;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneKind {
    LeftSidebar,
    CenterStage,
    RightSidebar,
}

struct PaneGridLayout {
    state: pane_grid::State<PaneKind>,
    left_split: Option<pane_grid::Split>,
    right_split: Option<pane_grid::Split>,
}

fn build_pane_grid_layout(
    left_mode: LeftSidebarMode,
    right_mode: RightSidebarMode,
    left_ratio: f32,
    right_ratio: f32,
) -> PaneGridLayout {
    use pane_grid::Axis;

    let (mut state, center_pane) = pane_grid::State::new(PaneKind::CenterStage);
    let mut left_split = None;
    let mut right_split = None;

    if left_mode != LeftSidebarMode::Hidden {
        if let Some((left_pane, split)) =
            state.split(Axis::Vertical, center_pane, PaneKind::LeftSidebar)
        {
            state.swap(center_pane, left_pane);
            state.resize(split, clamp_ratio(left_ratio));
            left_split = Some(split);
        }
    }

    if right_mode != RightSidebarMode::Hidden {
        if let Some((_right_pane, split)) =
            state.split(Axis::Vertical, center_pane, PaneKind::RightSidebar)
        {
            state.resize(split, clamp_ratio(right_ratio));
            right_split = Some(split);
        }
    }

    PaneGridLayout {
        state,
        left_split,
        right_split,
    }
}

const DEFAULT_WINDOW_WIDTH: f32 = 1280.0;
const LEFT_SIDEBAR_WIDTH: f32 = 300.0;
const RIGHT_SIDEBAR_WIDTH: f32 = 200.0;

fn default_left_ratio() -> f32 {
    clamp_ratio(LEFT_SIDEBAR_WIDTH / DEFAULT_WINDOW_WIDTH)
}

fn default_right_ratio() -> f32 {
    let center_width = DEFAULT_WINDOW_WIDTH - LEFT_SIDEBAR_WIDTH - RIGHT_SIDEBAR_WIDTH;
    clamp_ratio(center_width / (center_width + RIGHT_SIDEBAR_WIDTH))
}

fn clamp_ratio(value: f32) -> f32 {
    value.clamp(0.05, 0.95)
}
