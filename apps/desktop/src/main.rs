use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

use iced::futures::channel::oneshot;
use iced::widget::image::Handle;
use iced::widget::{Row, column};
use iced::{Element, Length, Task};

mod business;
use business::cache::compare_cache_to_fs;
use business::workspace::{WorkspaceModel, WorkspaceScanResult};

mod components;
use components::center_stage::center_stage;
use components::control_panel_bottom::control_panel_bottom;
use components::control_panel_top::control_panel_top;
use components::divider::divider;
use components::sidebar_left::{LeftSidebarMode, sidebar_left};
use components::sidebar_right::{RightSidebarMode, sidebar_right};

use io::catalog::ImageDO;
use io::catalog::catalog::{CATALOG_FILE_NAME, CATALOG_FOLDER_NAME, Catalog};
use io::catalog::catalog_error::CatalogError;
use io::image_files::helpers::{FolderScanResult, scan_folder_images};
use previews::preview_generation::{
    PREVIEW_FILE_TYPE, PreviewGenerationError, generate_preview_for_image,
};
use rfd::FileDialog;

pub enum ViewMode {
    Library,
    Develop,
    NoCatalog,
}

pub struct NavigatorState {
    expanded: HashSet<PathBuf>,
    selected: Option<PathBuf>,
    context_menu_root: Option<PathBuf>,
    context_menu_open: bool,
}

pub struct WorkspaceState {
    pub model: WorkspaceModel,

    // Imported roots currently being scanned (initial load + reselection refreshes).
    pub roots_scanning: HashSet<PathBuf>,

    // Persistent preview payload cache (hash -> Preview).
    pub preview_cache: HashMap<String, Preview>,

    // Current render set for selected folder (kept for compatibility with existing center stage).
    pub previews: HashMap<String, Preview>,

    pub handle_to_missing_preview_placeholder: Handle,
}

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

#[derive(Debug, Clone)]
pub struct SelectionDiffData {
    request_id: u64,
    selected_path: PathBuf,
    images_to_add_to_catalog: Vec<PathBuf>,
    catalog_image_dos_to_delete: Vec<ImageDO>,
}

pub struct App {
    pub left_sidebar_mode: LeftSidebarMode,
    pub right_sidebar_mode: RightSidebarMode,
    pub view_mode: ViewMode,
    pub catalog: Option<Catalog>,
    pub imported_dirs: Vec<PathBuf>,
    pub navigator_state: NavigatorState,
    pub workspace_state: WorkspaceState,
    selection_request_seq: u64,
    active_selection_request_id: Option<u64>,
}

static APP_START: OnceLock<Instant> = OnceLock::new();

fn startup_elapsed_ms() -> u128 {
    APP_START.get_or_init(Instant::now).elapsed().as_millis()
}

fn startup_log(event: &str) {
    println!("[Startup +{}ms] {}", startup_elapsed_ms(), event);
}

// init state
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

    fn refresh_selected_previews_from_cache(&mut self) {
        self.workspace_state.previews.clear();

        let Some(selected) = self.navigator_state.selected.as_ref() else {
            return;
        };

        for key in self
            .workspace_state
            .model
            .preview_keys_for_selected_folder(selected)
        {
            if let Some(preview) = self.workspace_state.preview_cache.get(&key) {
                self.workspace_state.previews.insert(key, preview.clone());
            }
        }
    }

    fn to_workspace_scan_result(scan_result: &FolderScanResult) -> WorkspaceScanResult {
        WorkspaceScanResult::new(
            scan_result.all_image_paths.clone(),
            scan_result.all_folders.clone(),
            scan_result.direct_image_counts.clone(),
        )
    }

    fn build_preview_from_image_do(catalog: &Catalog, image_do: &ImageDO) -> Preview {
        let path = catalog.root().join(catalog.cache_dir()).join(format!(
            "{}.{}",
            image_do.hash,
            PREVIEW_FILE_TYPE.get_file_extension()
        ));

        Preview {
            path_to_original: PathBuf::from(&image_do.path),
            hash: image_do.hash.clone(),
            img_handle: if path.exists() {
                Some(Handle::from_path(path.clone()))
            } else {
                None
            },
            preview_state: if path.exists() {
                PreviewState::Ok
            } else {
                PreviewState::OriginalMissing
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    LeftSidebarClicked(LeftSidebarMode),
    RightSidebarClicked(RightSidebarMode),
    CreateCatalog,
    SelectCatalog,
    CatalogLoadAttempted(Result<Catalog, CatalogError>),
    CatalogLoaded,
    NavigatorCollapseAll,
    ImportDirectory,
    LoadImportedDirectories,
    ImportedDirectoriesLoadAttempted(Result<Vec<PathBuf>, CatalogError>),
    ErrorMessage(String),
    ToggleDirectory(PathBuf),
    SelectDirectory(PathBuf),
    OpenRootContextMenu(PathBuf),
    CloseRootContextMenu,
    RefreshImportedRoot(PathBuf),
    WorkspaceRootScanned((PathBuf, FolderScanResult)),
    SelectionCatalogLoaded(Result<(u64, PathBuf, Vec<ImageDO>), CatalogError>),
    SelectionDiffComputed(SelectionDiffData),
    PreviewDataLoadedForImage(Preview),
    PreviewGenerated(Result<ImageDO, PreviewGenerationError>),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LeftSidebarClicked(mode) => {
                if self.left_sidebar_mode != LeftSidebarMode::Hidden
                    && self.left_sidebar_mode == mode
                {
                    self.left_sidebar_mode = LeftSidebarMode::Hidden;
                } else {
                    self.left_sidebar_mode = mode;
                }
                Task::none()
            }
            Message::RightSidebarClicked(mode) => {
                if self.right_sidebar_mode != RightSidebarMode::Hidden
                    && self.right_sidebar_mode == mode
                {
                    self.right_sidebar_mode = RightSidebarMode::Hidden;
                } else {
                    self.right_sidebar_mode = mode;
                }
                Task::none()
            }
            Message::CreateCatalog => {
                if let Some(path) = FileDialog::new().pick_folder() {
                    Task::perform(Catalog::create(path), Message::CatalogLoadAttempted)
                } else {
                    println!("FileDialog canceled");
                    Task::none()
                }
            }
            Message::SelectCatalog => {
                if let Some(path) = FileDialog::new().pick_folder() {
                    Task::perform(Catalog::load(path), Message::CatalogLoadAttempted)
                } else {
                    println!("FileDialog canceled");
                    Task::none()
                }
            }
            Message::CatalogLoadAttempted(result) => match result {
                Ok(catalog) => {
                    startup_log("CatalogLoadAttempted: success");
                    self.catalog = Some(catalog);
                    Task::perform(async {}, |_| Message::CatalogLoaded)
                }
                Err(e) => {
                    startup_log("CatalogLoadAttempted: error");
                    eprintln!("Error loading catalog: {}", e);
                    Task::none()
                }
            },
            Message::CatalogLoaded => {
                startup_log("CatalogLoaded message received");
                self.view_mode = ViewMode::Library;
                startup_log("Dispatching LoadImportedDirectories");
                Task::perform(async {}, |_| Message::LoadImportedDirectories)
            }
            Message::NavigatorCollapseAll => {
                self.navigator_state.expanded.clear();
                Task::none()
            }
            Message::ImportDirectory => {
                let Some(catalog) = self.catalog.clone() else {
                    println!("Cannot import directory: no catalog loaded");
                    return Task::none();
                };

                if let Some(path) = FileDialog::new().pick_folder() {
                    let catalog_clone = catalog.clone();
                    Task::perform(
                        async move { catalog_clone.import_directory(path.clone()).await },
                        |res| match res {
                            Ok(_) => Message::LoadImportedDirectories,
                            Err(_e) => Message::ErrorMessage("Failed to import directory".into()),
                        },
                    )
                } else {
                    println!("FileDialog canceled");
                    Task::none()
                }
            }
            Message::LoadImportedDirectories => {
                startup_log("LoadImportedDirectories started");
                if let Some(catalog) = &self.catalog {
                    let catalog_clone = catalog.clone();
                    Task::perform(
                        async move { catalog_clone.get_imported_directories().await },
                        Message::ImportedDirectoriesLoadAttempted,
                    )
                } else {
                    startup_log("LoadImportedDirectories skipped (no catalog)");
                    Task::none()
                }
            }
            Message::ImportedDirectoriesLoadAttempted(result) => match result {
                Ok(paths) => {
                    startup_log(&format!(
                        "ImportedDirectoriesLoadAttempted: success ({} roots)",
                        paths.len()
                    ));
                    self.imported_dirs = paths.clone();

                    self.workspace_state.model.clear();
                    self.workspace_state.model.root_folders = paths.clone();
                    self.workspace_state.preview_cache.clear();
                    self.workspace_state.previews.clear();
                    self.workspace_state.roots_scanning = paths.iter().cloned().collect();

                    startup_log("Dispatching root scan tasks");

                    let scan_tasks: Vec<Task<Message>> = paths
                        .iter()
                        .cloned()
                        .map(|root| {
                            Task::perform(
                                async move {
                                    let (tx, rx) = oneshot::channel();
                                    std::thread::spawn(move || {
                                        let scan_result = scan_folder_images(root.clone());
                                        let _ = tx.send((root, scan_result));
                                    });

                                    rx.await.expect("Thread panicked or channel closed")
                                },
                                Message::WorkspaceRootScanned,
                            )
                        })
                        .collect();

                    Task::batch(scan_tasks)
                }
                Err(e) => {
                    startup_log("ImportedDirectoriesLoadAttempted: error");
                    println!(
                        "Error while loading imported directories from catalog: {0:?}",
                        e
                    );
                    Task::none()
                }
            },
            Message::ErrorMessage(_msg) => Task::none(),
            Message::ToggleDirectory(path) => {
                self.navigator_state.context_menu_open = false;
                self.navigator_state.context_menu_root = None;

                if self.navigator_state.expanded.contains(&path) {
                    self.navigator_state.expanded.remove(&path);
                } else {
                    self.navigator_state.expanded.insert(path);
                }
                Task::none()
            }
            Message::OpenRootContextMenu(path) => {
                self.navigator_state.context_menu_root = Some(path);
                self.navigator_state.context_menu_open = true;
                Task::none()
            }
            Message::CloseRootContextMenu => {
                self.navigator_state.context_menu_open = false;
                self.navigator_state.context_menu_root = None;
                Task::none()
            }
            Message::RefreshImportedRoot(root) => {
                self.navigator_state.context_menu_open = false;
                self.navigator_state.context_menu_root = None;
                self.workspace_state.roots_scanning.insert(root.clone());

                Task::perform(
                    async move {
                        let (tx, rx) = oneshot::channel();
                        std::thread::spawn(move || {
                            let scan_result = scan_folder_images(root.clone());
                            let _ = tx.send((root, scan_result));
                        });

                        rx.await.expect("Thread panicked or channel closed")
                    },
                    Message::WorkspaceRootScanned,
                )
            }
            Message::WorkspaceRootScanned((root, scan_result)) => {
                startup_log(&format!(
                    "WorkspaceRootScanned: {} ({} folders, {} images)",
                    root.to_string_lossy(),
                    scan_result.all_folders.len(),
                    scan_result.all_image_paths.len()
                ));
                self.workspace_state
                    .model
                    .apply_root_scan_update(&root, Self::to_workspace_scan_result(&scan_result));
                self.workspace_state.roots_scanning.remove(&root);

                if self.workspace_state.roots_scanning.is_empty() {
                    startup_log("All root scans finished");
                }

                Task::none()
            }
            Message::SelectDirectory(path) => {
                self.navigator_state.context_menu_open = false;
                self.navigator_state.context_menu_root = None;

                if self.navigator_state.selected.as_ref() == Some(&path) {
                    self.navigator_state.selected = None;
                    self.workspace_state.previews.clear();
                    self.selection_request_seq = self.selection_request_seq.wrapping_add(1);
                    self.active_selection_request_id = None;
                    return Task::none();
                }

                self.navigator_state.selected = Some(path.clone());
                self.selection_request_seq = self.selection_request_seq.wrapping_add(1);
                let request_id = self.selection_request_seq;
                self.active_selection_request_id = Some(request_id);
                self.refresh_selected_previews_from_cache();

                let Some(catalog) = self.catalog.clone() else {
                    return Task::none();
                };

                Task::perform(
                    async move {
                        let selected_path = path.clone();
                        let image_dos = catalog.get_all_image_dos_for_path(&selected_path).await?;
                        Ok((request_id, selected_path, image_dos))
                    },
                    Message::SelectionCatalogLoaded,
                )
            }
            Message::SelectionCatalogLoaded(load_result) => match load_result {
                Ok((request_id, selected_path, image_dos)) => {
                    if self.active_selection_request_id != Some(request_id) {
                        return Task::none();
                    }

                    if self.navigator_state.selected.as_ref() != Some(&selected_path) {
                        return Task::none();
                    }

                    for image_do in &image_dos {
                        self.workspace_state.model.upsert_preview_key(
                            image_do.hash.clone(),
                            PathBuf::from(&image_do.path),
                        );
                    }

                    self.refresh_selected_previews_from_cache();

                    let mut tasks: Vec<Task<Message>> = Vec::new();

                    if let Some(catalog) = &self.catalog {
                        let catalog_clone = catalog.clone();

                        for image_do in &image_dos {
                            let hash = image_do.hash.clone();

                            if self.workspace_state.preview_cache.contains_key(&hash) {
                                continue;
                            }

                            let image_do = image_do.clone();
                            let catalog_for_task = catalog_clone.clone();
                            tasks.push(Task::perform(
                                async move {
                                    Self::build_preview_from_image_do(&catalog_for_task, &image_do)
                                },
                                Message::PreviewDataLoadedForImage,
                            ));
                        }
                    }

                    let selected_path_for_diff = selected_path.clone();
                    let image_dos_for_diff = image_dos.clone();

                    tasks.push(Task::perform(
                        async move {
                            let (tx, rx) = oneshot::channel();
                            std::thread::spawn(move || {
                                let selected_scan =
                                    scan_folder_images(selected_path_for_diff.clone());

                                let (images_to_add_to_catalog, catalog_image_dos_to_delete) =
                                    compare_cache_to_fs(
                                        selected_scan.all_image_paths,
                                        image_dos_for_diff,
                                    );

                                let _ = tx.send(SelectionDiffData {
                                    request_id,
                                    selected_path: selected_path_for_diff,
                                    images_to_add_to_catalog,
                                    catalog_image_dos_to_delete,
                                });
                            });

                            rx.await.expect("Thread panicked or channel closed")
                        },
                        Message::SelectionDiffComputed,
                    ));

                    Task::batch(tasks)
                }
                Err(e) => {
                    println!("[Selection] Failed to load data for selected folder: {}", e);
                    Task::none()
                }
            },
            Message::SelectionDiffComputed(diff_data) => {
                if self.active_selection_request_id != Some(diff_data.request_id) {
                    return Task::none();
                }

                for image_do in diff_data.catalog_image_dos_to_delete {
                    if let Some(preview) =
                        self.workspace_state.preview_cache.get_mut(&image_do.hash)
                    {
                        preview.preview_state = PreviewState::OriginalMissing;
                    }

                    if let Some(preview) = self.workspace_state.previews.get_mut(&image_do.hash) {
                        if preview
                            .path_to_original
                            .starts_with(&diff_data.selected_path)
                        {
                            preview.preview_state = PreviewState::OriginalMissing;
                        }
                    }
                }

                let mut add_tasks: Vec<Task<Message>> = Vec::new();
                if let Some(catalog) = &self.catalog {
                    let catalog_clone = catalog.clone();

                    for path in diff_data.images_to_add_to_catalog {
                        let catalog_for_task = catalog_clone.clone();
                        add_tasks.push(Task::perform(
                            async move {
                                generate_preview_for_image(path, &catalog_for_task, false).await
                            },
                            Message::PreviewGenerated,
                        ));
                    }
                }

                Task::batch(add_tasks)
            }
            Message::PreviewGenerated(result) => match result {
                Ok(image_do) => {
                    if let Some(catalog) = &self.catalog {
                        let preview = Self::build_preview_from_image_do(catalog, &image_do);

                        self.workspace_state.model.upsert_preview_key(
                            image_do.hash.clone(),
                            PathBuf::from(&image_do.path),
                        );

                        self.workspace_state
                            .preview_cache
                            .insert(image_do.hash.clone(), preview.clone());

                        if let Some(selected) = self.navigator_state.selected.as_ref() {
                            if preview.path_to_original.starts_with(selected) {
                                self.workspace_state.previews.insert(image_do.hash, preview);
                            }
                        }
                    }

                    Task::none()
                }
                Err(e) => {
                    println!("[Preview Generation] Failed with error: {}", e);
                    Task::none()
                }
            },
            Message::PreviewDataLoadedForImage(preview) => {
                let hash = preview.hash.clone();

                self.workspace_state
                    .model
                    .upsert_preview_key(hash.clone(), preview.path_to_original.clone());

                self.workspace_state
                    .preview_cache
                    .insert(hash.clone(), preview.clone());

                if let Some(selected) = self.navigator_state.selected.as_ref() {
                    if preview.path_to_original.starts_with(selected) {
                        self.workspace_state.previews.insert(hash, preview);
                    }
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
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

    fn theme(&self) -> iced::Theme {
        let palette = iced::theme::Palette {
            background: iced::color!(0x1e1e24),
            text: iced::color!(0xb0b0b5),
            primary: iced::color!(0x4A90E2),
            success: iced::color!(0x4CAF50),
            warning: iced::color!(0xFFC107),
            danger: iced::color!(0xF44336),
        };

        iced::Theme::custom_with_fn("Maelstrom Dark", palette, |palette| {
            let mut extended = iced::theme::palette::Extended::generate(palette);

            extended.background.base.color = iced::color!(0x1d1e24);
            extended.background.weak.color = iced::color!(0x23252b);
            extended.background.strong.color = iced::color!(0x2a2d34);

            extended
        })
    }
}

fn main() -> iced::Result {
    let window_settings = iced::window::Settings {
        platform_specific: iced::window::settings::PlatformSpecific {
            title_hidden: false,
            titlebar_transparent: true,
            fullsize_content_view: true,
        },
        ..Default::default()
    };

    iced::application(App::new, App::update, App::view)
        .theme(App::theme)
        .title("Maelstrom")
        .window(window_settings)
        .run()
}
