use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

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
}

pub struct WorkspaceState {
    pub model: WorkspaceModel,

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
pub struct SelectionLoadData {
    selected_path: PathBuf,
    root_path: PathBuf,
    scan_result: FolderScanResult,
    image_dos: Vec<ImageDO>,
}

pub struct App {
    pub left_sidebar_mode: LeftSidebarMode,
    pub right_sidebar_mode: RightSidebarMode,
    pub view_mode: ViewMode,
    pub catalog: Option<Catalog>,
    pub imported_dirs: Vec<PathBuf>,
    pub navigator_state: NavigatorState,
    pub workspace_state: WorkspaceState,
}

// init state
impl App {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            left_sidebar_mode: LeftSidebarMode::Navigator,
            right_sidebar_mode: RightSidebarMode::Hidden,
            view_mode: ViewMode::NoCatalog,
            catalog: None,
            imported_dirs: Vec::new(),
            navigator_state: NavigatorState {
                expanded: HashSet::new(),
                selected: None,
            },
            workspace_state: WorkspaceState {
                model: WorkspaceModel::default(),
                preview_cache: HashMap::new(),
                previews: HashMap::new(),
                handle_to_missing_preview_placeholder: Handle::from_bytes(
                    include_bytes!("../assets/static/image_missing.png").to_vec(),
                ),
            },
        };

        let config_base = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("maelstrom");

        if !config_base.exists() {
            println!("User config dir doesnt exists at {:?}", config_base);
            return (app, Task::none());
        }

        let catalog_root = config_base.join(CATALOG_FOLDER_NAME);

        let startup_task = if catalog_root.join(CATALOG_FILE_NAME).exists() {
            Task::perform(
                Catalog::load(catalog_root.clone()),
                Message::CatalogLoadAttempted,
            )
        } else {
            println!("default catalog not found, creating at: {:?}", catalog_root);
            Task::perform(
                Catalog::create(config_base.clone()),
                Message::CatalogLoadAttempted,
            )
        };

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

    fn find_root_for_selected(imported_dirs: &[PathBuf], selected: &Path) -> Option<PathBuf> {
        imported_dirs
            .iter()
            .filter(|root| selected.starts_with(root))
            .max_by_key(|root| root.components().count())
            .cloned()
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
    WorkspaceRootScanned((PathBuf, FolderScanResult)),
    SelectionDataReady(Result<SelectionLoadData, CatalogError>),
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
                    self.catalog = Some(catalog.clone());
                    let catalog_for_task = catalog.clone();

                    Task::perform(
                        async move {
                            catalog_for_task.print_metadata().await.ok();
                        },
                        |_| Message::CatalogLoaded,
                    )
                }
                Err(e) => {
                    eprintln!("Error loading catalog: {}", e);
                    Task::none()
                }
            },
            Message::CatalogLoaded => {
                self.view_mode = ViewMode::Library;
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
                        async move {
                            let result = catalog_clone.import_directory(path.clone()).await;
                            if result.is_ok() {
                                catalog_clone.print_metadata().await.ok();
                            }
                            result
                        },
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
                if let Some(catalog) = &self.catalog {
                    let catalog_clone = catalog.clone();
                    Task::perform(
                        async move { catalog_clone.get_imported_directories().await },
                        Message::ImportedDirectoriesLoadAttempted,
                    )
                } else {
                    Task::none()
                }
            }
            Message::ImportedDirectoriesLoadAttempted(result) => match result {
                Ok(paths) => {
                    self.imported_dirs = paths.clone();

                    self.workspace_state.model.clear();
                    self.workspace_state.preview_cache.clear();
                    self.workspace_state.previews.clear();

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
                    println!(
                        "Error while loading imported directories from catalog: {0:?}",
                        e
                    );
                    Task::none()
                }
            },
            Message::ErrorMessage(_msg) => Task::none(),
            Message::ToggleDirectory(path) => {
                if self.navigator_state.expanded.contains(&path) {
                    self.navigator_state.expanded.remove(&path);
                } else {
                    self.navigator_state.expanded.insert(path);
                }
                Task::none()
            }
            Message::WorkspaceRootScanned((root, scan_result)) => {
                self.workspace_state
                    .model
                    .apply_root_scan_update(&root, Self::to_workspace_scan_result(&scan_result));
                Task::none()
            }
            Message::SelectDirectory(path) => {
                if self.navigator_state.selected.as_ref() == Some(&path) {
                    self.navigator_state.selected = None;
                    self.workspace_state.previews.clear();
                    return Task::none();
                }

                self.navigator_state.selected = Some(path.clone());
                self.refresh_selected_previews_from_cache();

                let Some(catalog) = self.catalog.clone() else {
                    return Task::none();
                };

                let Some(root_path) = Self::find_root_for_selected(&self.imported_dirs, &path)
                else {
                    println!(
                        "[Selection] No imported root found for selected folder: {:?}",
                        path
                    );
                    return Task::none();
                };

                Task::perform(
                    async move {
                        let selected_path = path.clone();

                        let (tx, rx) = oneshot::channel();
                        let root_for_thread = root_path.clone();
                        std::thread::spawn(move || {
                            let scan_result = scan_folder_images(root_for_thread);
                            let _ = tx.send(scan_result);
                        });

                        let scan_result = rx.await.expect("Thread panicked or channel closed");
                        let image_dos = catalog.get_all_image_dos_for_path(&selected_path).await?;

                        Ok(SelectionLoadData {
                            selected_path,
                            root_path,
                            scan_result,
                            image_dos,
                        })
                    },
                    Message::SelectionDataReady,
                )
            }
            Message::SelectionDataReady(load_result) => match load_result {
                Ok(data) => {
                    self.workspace_state.model.apply_root_scan_update(
                        &data.root_path,
                        Self::to_workspace_scan_result(&data.scan_result),
                    );

                    for image_do in &data.image_dos {
                        self.workspace_state.model.upsert_preview_key(
                            image_do.hash.clone(),
                            PathBuf::from(&image_do.path),
                        );
                    }

                    self.refresh_selected_previews_from_cache();

                    let selected_fs_images: Vec<PathBuf> = data
                        .scan_result
                        .all_image_paths
                        .iter()
                        .filter(|p| p.starts_with(&data.selected_path))
                        .cloned()
                        .collect();

                    let (images_to_add_to_catalog, catalog_image_dos_to_delete) =
                        compare_cache_to_fs(selected_fs_images, data.image_dos.clone());

                    let mut tasks: Vec<Task<Message>> = Vec::new();

                    for image_do in catalog_image_dos_to_delete {
                        if let Some(preview) =
                            self.workspace_state.preview_cache.get_mut(&image_do.hash)
                        {
                            preview.preview_state = PreviewState::OriginalMissing;
                        }

                        if let Some(preview) = self.workspace_state.previews.get_mut(&image_do.hash)
                        {
                            preview.preview_state = PreviewState::OriginalMissing;
                        }
                    }

                    if let Some(catalog) = &self.catalog {
                        let catalog_clone = catalog.clone();

                        for image_do in data.image_dos {
                            let hash = image_do.hash.clone();

                            if self.workspace_state.preview_cache.contains_key(&hash) {
                                continue;
                            }

                            let catalog_for_task = catalog_clone.clone();
                            tasks.push(Task::perform(
                                async move {
                                    println!(
                                        "[Preview Loading] Loading preview data for {}",
                                        image_do.path
                                    );
                                    Self::build_preview_from_image_do(&catalog_for_task, &image_do)
                                },
                                Message::PreviewDataLoadedForImage,
                            ));
                        }

                        for path in images_to_add_to_catalog {
                            let catalog_for_task = catalog_clone.clone();
                            tasks.push(Task::perform(
                                async move {
                                    generate_preview_for_image(path, &catalog_for_task, false).await
                                },
                                Message::PreviewGenerated,
                            ));
                        }
                    }

                    Task::batch(tasks)
                }
                Err(e) => {
                    println!("[Selection] Failed to load data for selected folder: {}", e);
                    Task::none()
                }
            },
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
