use iced::widget::{Row, column};
use iced::{Element, Length};

mod components;
use components::center_stage::center_stage;
use components::control_panel_bottom::control_panel_bottom;
use components::control_panel_top::control_panel_top;
use components::divider::divider;
use components::sidebar_left::{LeftSidebarMode, sidebar_left};
use components::sidebar_right::{RightSidebarMode, sidebar_right};

pub enum ViewMode {
    Library,
    Develop,
    NoCatalog,
}

pub struct App {
    pub left_sidebar_mode: LeftSidebarMode,
    pub right_sidebar_mode: RightSidebarMode,
    pub view_mode: ViewMode,
}

// init state
impl Default for App {
    fn default() -> Self {
        Self {
            left_sidebar_mode: LeftSidebarMode::Navigator,
            right_sidebar_mode: RightSidebarMode::Hidden,
            view_mode: ViewMode::NoCatalog,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    LeftSidebarClicked(LeftSidebarMode),
    RightSidebarClicked(RightSidebarMode),
    CreateCatalog,
    ImportCatalog,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::LeftSidebarClicked(mode) => {
                if self.left_sidebar_mode != LeftSidebarMode::Hidden
                    && self.left_sidebar_mode == mode
                {
                    self.left_sidebar_mode = LeftSidebarMode::Hidden;
                } else {
                    self.left_sidebar_mode = mode;
                }
            }
            Message::RightSidebarClicked(mode) => {
                if self.right_sidebar_mode != RightSidebarMode::Hidden
                    && self.right_sidebar_mode == mode
                {
                    self.right_sidebar_mode = RightSidebarMode::Hidden;
                } else {
                    self.right_sidebar_mode = mode;
                }
            }
            Message::CreateCatalog => {
                println!("Click create")
            }
            Message::ImportCatalog => {
                println!("Click import")
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
