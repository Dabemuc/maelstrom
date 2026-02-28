use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

use iced::widget::image::Handle;
use iced::widget::{Row, column};
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
                handle_to_missing_preview_placeholder: Handle::from_bytes(
                    include_bytes!("../assets/static/image_missing.png").to_vec(),
                ),
            },
            selection_request_seq: 0,
            active_selection_request_id: None,
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
        let mut main_content = Row::new().height(Length::Fill);

        if self.left_sidebar_mode != LeftSidebarMode::Hidden {
            main_content = main_content.push(sidebar_left(self));
        }

        main_content = main_content.push(center_stage(self));

        if self.right_sidebar_mode != RightSidebarMode::Hidden {
            main_content = main_content.push(sidebar_right(self));
        }

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
}
