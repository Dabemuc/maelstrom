use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use iced::widget::{Row, column};
use iced::{Element, Length, Task};

mod components;
use components::center_stage::center_stage;
use components::control_panel_bottom::control_panel_bottom;
use components::control_panel_top::control_panel_top;
use components::divider::divider;
use components::sidebar_left::{LeftSidebarMode, sidebar_left};
use components::sidebar_right::{RightSidebarMode, sidebar_right};

use io::catalog::catalog::Catalog;
use io::catalog::catalog_error::CatalogError;
use io::image_files::helpers::count_images_in_folder;
use rfd::FileDialog;

pub enum ViewMode {
    Library,
    Develop,
    NoCatalog,
}

pub struct NavigatorState {
    expanded: HashSet<PathBuf>,
    selected: Option<PathBuf>,
    image_counts: HashMap<PathBuf, u32>,
}

pub struct App {
    pub left_sidebar_mode: LeftSidebarMode,
    pub right_sidebar_mode: RightSidebarMode,
    pub view_mode: ViewMode,
    pub catalog: Option<Catalog>,
    pub imported_dirs: Vec<PathBuf>,
    pub navigator_state: NavigatorState,
}

// init state
impl Default for App {
    fn default() -> Self {
        Self {
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
    ImportDirectory,
    LoadImportedDirectories,
    ImportedDirectoriesLoadAttempted(Result<Vec<PathBuf>, CatalogError>),
    ErrorMessage(String),
    ToggleDirectory(PathBuf),
    SelectDirectory(PathBuf),
    ImageCountResult((PathBuf, u32)),
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
                if let Some(path) = FileDialog::new()
                    .add_filter("Maelstrom Catalog File", &["mcat"])
                    .pick_file()
                {
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
                                Task::perform(
                                    {
                                        let path = path.clone(); // copy or clone as needed
                                        async move {
                                            let count = count_images_in_folder(path.clone());
                                            (path, count) // return a tuple of (PathBuf, u32)
                                        }
                                    },
                                    Message::ImageCountResult, // this variant must accept (PathBuf, u32)
                                )
                            })
                            .collect();
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
                    self.navigator_state.selected = Some(path);
                } else {
                    if self.navigator_state.selected.as_ref().unwrap() == &path {
                        self.navigator_state.selected = None;
                    } else {
                        self.navigator_state.selected = Some(path)
                    }
                }
                
                println!("Selected: {:?}", self.navigator_state.selected);

                Task::none()
            }
            Message::ImageCountResult((path, count)) => {
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
    iced::application(App::default, App::update, App::view)
        .theme(App::theme)
        .title("Maelstrom")
        .window(window_settings)
        .run()
}
