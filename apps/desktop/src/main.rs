use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use iced::futures::channel::oneshot;
use iced::widget::image::Handle;
use iced::widget::{Row, column};
use iced::{Element, Length, Task};

mod business;
use business::cache::compare_cache_to_fs;

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
use io::image_files::helpers::collect_images_in_folder;
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
    image_counts: HashMap<PathBuf, usize>,
}

pub struct WorkspaceState {
    previews: HashMap<String, Preview>,
    handle_to_missing_preview_placeholder: Handle,
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
        // handle ignored
        self.preview_state.hash(state);
    }
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
        // 1. Initialize default state
        let app = Self {
            left_sidebar_mode: LeftSidebarMode::Navigator,
            right_sidebar_mode: RightSidebarMode::Hidden,
            view_mode: ViewMode::NoCatalog,
            catalog: None,
            imported_dirs: Vec::new(),
            navigator_state: NavigatorState {
                expanded: HashSet::new(),
                selected: None,
                image_counts: HashMap::new(),
            },
            workspace_state: WorkspaceState {
                previews: HashMap::new(),
                handle_to_missing_preview_placeholder: Handle::from_bytes(
                    include_bytes!("../assets/static/image_missing.png").to_vec(),
                ),
            },
        };

        // 2. Determine base config directory
        let config_base = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("maelstrom");

        // Ensure the base directory exists
        if !config_base.exists() {
            println!("User config dir doesnt exists at {:?}", config_base);
            return (app, Task::none());
        }

        // 3. Compute the catalog root folder
        let catalog_root = config_base.join(CATALOG_FOLDER_NAME);

        // 4. Prepare the startup task
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
    ImageCountResult((PathBuf, usize)),
    ImageDOsLoadedForPath(Result<Vec<ImageDO>, CatalogError>),
    PreviewDataLoadedForImage(Preview),
    CacheRefreshDone((Vec<PathBuf>, Vec<ImageDO>)),
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
                println!("Click create");
                if let Some(path) = FileDialog::new().pick_folder() {
                    return Task::perform(Catalog::create(path), Message::CatalogLoadAttempted);
                } else {
                    println!("FileDialog canceled");
                    Task::none()
                }
            }
            Message::SelectCatalog => {
                println!("Click select");
                if let Some(path) = FileDialog::new().pick_folder() {
                    return Task::perform(Catalog::load(path), Message::CatalogLoadAttempted);
                } else {
                    println!("FileDialog canceled");
                    Task::none()
                }
            }
            Message::CatalogLoadAttempted(result) => {
                match result {
                    Ok(catalog) => {
                        // Store catalog in state
                        self.catalog = Some(catalog.clone());

                        // Clone catalog for async task
                        let catalog_for_task = catalog.clone();

                        // Return a Task that prints metadata asynchronously
                        return Task::perform(
                            async move {
                                // Any errors ignored here, just printing
                                catalog_for_task.print_metadata().await.ok();
                            },
                            |_| Message::CatalogLoaded, // Dummy callback, owns a clone
                        );
                    }
                    Err(e) => {
                        eprintln!("Error loading catalog: {}", e);
                        Task::none()
                    }
                }
            }
            Message::CatalogLoaded => {
                self.view_mode = ViewMode::Library;

                let load_dirs_task = Task::perform(async {}, |_| Message::LoadImportedDirectories);

                load_dirs_task.chain(Task::none())
            }
            Message::NavigatorCollapseAll => {
                self.navigator_state.expanded.clear();

                Task::none()
            }
            Message::ImportDirectory => {
                println!("Click import");

                // Ensure catalog is loaded
                let catalog = if let Some(c) = self.catalog.clone() {
                    c
                } else {
                    println!("Cannot import directory: no catalog loaded");
                    return Task::none();
                };

                // Pick a folder
                if let Some(path) = FileDialog::new().pick_folder() {
                    let catalog_clone = catalog.clone();
                    return Task::perform(
                        async move {
                            // Import the folder
                            let result = catalog_clone.import_directory(path.clone()).await;

                            // Print metadata after successful import
                            if result.is_ok() {
                                catalog_clone.print_metadata().await.ok();
                            }

                            result
                        },
                        |res| match res {
                            Ok(_) => Message::LoadImportedDirectories,
                            Err(e) => {
                                eprintln!("Failed to import directory: {}", e);
                                Message::ErrorMessage(format!("Failed to import directory"))
                            }
                        },
                    );
                }

                println!("FileDialog canceled");
                Task::none()
            }
            Message::LoadImportedDirectories => {
                if let Some(catalog) = &self.catalog {
                    let catalog_clone = catalog.clone();
                    return Task::perform(
                        async move { catalog_clone.get_imported_directories().await },
                        Message::ImportedDirectoriesLoadAttempted,
                    );
                }
                Task::none()
            }
            Message::ImportedDirectoriesLoadAttempted(result) => {
                match result {
                    Ok(paths) => {
                        self.imported_dirs = paths.clone();
                        println!("Successfully loaded imported directories into state");

                        // Start image counting
                        let counting_tasks: Vec<Task<Message>> = paths
                            .iter()
                            .map(|path| {
                                let path = path.clone();
                                Task::perform(
                                    async move {
                                        println!("[Counting task] Starting count for {:?}", path);

                                        // Use oneshot to avoid blocking the async executor
                                        let (tx, rx) = oneshot::channel();
                                        std::thread::spawn(move || {
                                            let count =
                                                collect_images_in_folder(path.clone()).len();
                                            let _ = tx.send((path, count));
                                        });

                                        // Await asynchronously without blocking
                                        rx.await.expect("Thread panicked or channel closed")
                                    },
                                    Message::ImageCountResult,
                                )
                            })
                            .collect();

                        println!("[Counting task] Starting {} tasks", counting_tasks.len());

                        Task::batch(counting_tasks)
                    }
                    Err(e) => {
                        println!(
                            "Error while Loading imported directories from catalog: {0:?}",
                            e
                        );
                        Task::none()
                    }
                }
            }
            Message::ErrorMessage(_msg) => {
                // eventually show the message in a popup or smth
                Task::none()
            }
            Message::ToggleDirectory(path) => {
                if self.navigator_state.expanded.contains(&path) {
                    self.navigator_state.expanded.remove(&path);
                } else {
                    self.navigator_state.expanded.insert(path);
                }
                Task::none()
            }
            Message::SelectDirectory(path) => {
                if self.navigator_state.selected.is_none() {
                    self.navigator_state.selected = Some(path.clone());
                } else {
                    if self.navigator_state.selected.as_ref().unwrap() == &path {
                        self.navigator_state.selected = None;
                    } else {
                        self.navigator_state.selected = Some(path.clone())
                    }
                }

                println!("Selected: {:?}", self.navigator_state.selected);

                // For now clear previews. Later we could keep them
                self.workspace_state.previews.clear();

                if let Some(catalog) = &self.catalog {
                    let catalog_clone = catalog.clone();
                    let path_clone = path.clone();
                    // Task to fetch all preview catalog entries
                    let fetch_previews_task = Task::perform(
                        async move {
                            println!("[Preview Loading] Step 1: Loading image DOs from catalog");
                            catalog_clone.get_all_image_dos_for_path(path_clone).await
                        },
                        Message::ImageDOsLoadedForPath,
                    );

                    // Task to refresh Cache
                    let catalog_clone = catalog.clone();
                    let path_clone = path.clone();
                    let refresh_cache_task = Task::perform(
                        async move {
                            println!("[Cache Refresh] Step 1: Start cache refresh");

                            // Get all images in path and compare to entries in catalog
                            let paths_of_images_in_folder =
                                collect_images_in_folder(path_clone.clone());
                            let image_dos_in_catalog = catalog_clone
                                .get_all_image_dos_for_path(path_clone)
                                .await
                                .unwrap();
                            let (images_to_add_to_catalog, catalog_image_dos_to_delete) =
                                compare_cache_to_fs(
                                    paths_of_images_in_folder,
                                    image_dos_in_catalog,
                                );

                            println!(
                                "[Cache Refresh] Step 1: Found {} images not indexed in catalog and {} indexed images that are not found in the fs:",
                                images_to_add_to_catalog.len(),
                                catalog_image_dos_to_delete.len()
                            );

                            //perform adds/deletes
                            if images_to_add_to_catalog.len() > 0 {
                                println!(
                                    "[Cache Refresh] Step 1: Images not indexed: {:#?}",
                                    images_to_add_to_catalog
                                );
                            }
                            if catalog_image_dos_to_delete.len() > 0 {
                                println!(
                                    "[Cache Refresh] Step 1: Indexed images not in fs: {:#?}",
                                    catalog_image_dos_to_delete
                                );
                            }

                            (images_to_add_to_catalog, catalog_image_dos_to_delete)
                        },
                        Message::CacheRefreshDone,
                    );

                    Task::batch([fetch_previews_task, refresh_cache_task])
                } else {
                    Task::none()
                }
            }
            Message::ImageDOsLoadedForPath(load_result) => match load_result {
                Ok(image_dos) => {
                    println!(
                        "[Preview Loading] Step 1: Succesfully loaded {} image DOs from catalog",
                        image_dos.len()
                    );

                    // Next step: Schedule a task for each image to load Preview Data
                    if let Some(catalog) = &self.catalog {
                        let mut tasks = Vec::new();

                        for image_do in image_dos {
                            let catalog_clone = catalog.clone();

                            let task = Task::perform(
                                async move {
                                    println!(
                                        "[Preview Loading] Step 2: Loading preview data for {}",
                                        image_do.path
                                    );

                                    let path = catalog_clone
                                        .root()
                                        .join(catalog_clone.cache_dir())
                                        .join(format!(
                                            "{}.{}",
                                            image_do.hash,
                                            PREVIEW_FILE_TYPE.get_file_extension()
                                        ));

                                    Preview {
                                        path_to_original: PathBuf::from(image_do.path),
                                        hash: image_do.hash,
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
                                },
                                Message::PreviewDataLoadedForImage,
                            );

                            tasks.push(task);
                        }

                        // Run simultaniously
                        return Task::batch(tasks);
                    } else {
                        println!("[Preview Loading] Step 2: Failed to load previews. No Catalog");
                        Task::none()
                    }
                }
                Err(e) => {
                    println!(
                        "[Preview Loading] Step 1: Error while loading image DOs from catalog: {}",
                        e
                    );
                    Task::none()
                }
            },
            Message::CacheRefreshDone((images_to_add_to_catalog, catalog_image_dos_to_delete)) => {
                println!("[Cache Refresh] Step 1: Cache refresh done",);

                // adds
                println!(
                    "[Cache Refresh] Step 2: Performing {} adds",
                    images_to_add_to_catalog.len(),
                );
                let mut add_tasks: Vec<Task<Message>> = Vec::new();
                if let Some(catalog) = &self.catalog {
                    for path in images_to_add_to_catalog {
                        let catalog_clone = catalog.clone();

                        add_tasks.push(Task::perform(
                            async move {
                                generate_preview_for_image(
                                    path,
                                    &catalog_clone,
                                    false,
                                )
                                .await
                            },
                            Message::PreviewGenerated,
                        ));
                    }
                } else {
                    println!("[Cache Refresh] Step 2: Failed to perform adds. No catalog");
                }

                // deletes (For now we dont want to completely remove entry. Just set state)
                println!(
                    "[Cache Refresh] Step 2: Performing {} deletes",
                    catalog_image_dos_to_delete.len(),
                );
                for image_do in catalog_image_dos_to_delete {
                    if let Some(preview) = self.workspace_state.previews.get_mut(&image_do.hash) {
                        preview.preview_state = PreviewState::OriginalMissing;
                    } else {
                        println!(
                            "[Cache Refresh] Step 2: Failed to update preview state for imageDO {:?}",
                            image_do
                        );
                    }
                }

                return Task::batch(add_tasks);
            }
            Message::PreviewGenerated(result) => match result {
                Ok(image_do) => {
                    if let Some(catalog) = &self.catalog {
                        let catalog_clone = catalog.clone();
                        let path =
                            catalog_clone
                                .root()
                                .join(catalog_clone.cache_dir())
                                .join(format!(
                                    "{}.{}",
                                    image_do.hash,
                                    PREVIEW_FILE_TYPE.get_file_extension()
                                ));
                        self.workspace_state.previews.insert(
                            image_do.hash.clone(),
                            Preview {
                                path_to_original: PathBuf::from(image_do.path),
                                hash: image_do.hash,
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
                            },
                        );

                        Task::none()
                    } else {
                        Task::none()
                    }
                }
                Err(e) => {
                    println!("[Preview Generation] Failed with error: {}", e);
                    Task::none()
                }
            },
            Message::PreviewDataLoadedForImage(preview) => {
                println!(
                    "[Preview Loading] Step 2: Loaded preview data for image {}",
                    preview.path_to_original.to_str().unwrap()
                );
                self.workspace_state
                    .previews
                    .insert(preview.hash.clone(), preview);
                Task::none()
            }

            Message::ImageCountResult((path, count)) => {
                println!(
                    "[Counting task] Received result of {} for {}",
                    count,
                    path.to_str().unwrap()
                );
                self.navigator_state.image_counts.insert(path, count);

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
        // Base dark palette
        let palette = iced::theme::Palette {
            background: iced::color!(0x1e1e24), // Slate-ish dark hue
            text: iced::color!(0xb0b0b5),
            primary: iced::color!(0x4A90E2),
            success: iced::color!(0x4CAF50),
            warning: iced::color!(0xFFC107),
            danger: iced::color!(0xF44336),
        };

        iced::Theme::custom_with_fn("Maelstrom Dark", palette, |palette| {
            // Let iced generate the standard variations for buttons, hover states, etc.
            let mut extended = iced::theme::palette::Extended::generate(palette);

            // Override the backgrounds to be very close in luminance (Zed style)
            // Center Stage (Darkest)
            extended.background.base.color = iced::color!(0x1d1e24);
            // Sidebars (A tiny bit lighter)
            extended.background.weak.color = iced::color!(0x23252b);
            // Control Panel (A tiny bit lighter than sidebars)
            extended.background.strong.color = iced::color!(0x2a2d34);

            extended
        })
    }
}

fn main() -> iced::Result {
    // 1. Configure the window to push content into the titlebar
    let window_settings = iced::window::Settings {
        platform_specific: iced::window::settings::PlatformSpecific {
            title_hidden: false,
            titlebar_transparent: true,
            fullsize_content_view: true,
        },
        ..Default::default()
    };

    // 2. Launch the application
    iced::application(App::new, App::update, App::view)
        .theme(App::theme)
        .title("Maelstrom")
        .window(window_settings)
        .run()
}
