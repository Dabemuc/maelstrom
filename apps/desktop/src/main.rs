use iced::widget::{Row, column};
use iced::{Element, Length, run};

mod components;
use components::center_stage::center_stage;
use components::control_panel::control_panel;
use components::sidebar_left::sidebar_left;
use components::sidebar_right::sidebar_right;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftSidebarMode {
    FileNavigator,
    Collections,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightSidebarMode {
    Edit,
}

pub struct App {
    pub left_sidebar_visible: bool,
    pub right_sidebar_visible: bool,
    pub left_sidebar_mode: LeftSidebarMode,
    pub right_sidebar_mode: RightSidebarMode,
}

// init state
impl Default for App {
    fn default() -> Self {
        Self {
            left_sidebar_visible: true,
            right_sidebar_visible: true,
            left_sidebar_mode: LeftSidebarMode::FileNavigator,
            right_sidebar_mode: RightSidebarMode::Edit,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    ToggleLeftSidebar,
    ToggleRightSidebar,
    LeftSidebarClicked(LeftSidebarMode),
    RightSidebarClicked(RightSidebarMode),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleLeftSidebar => self.left_sidebar_visible = !self.left_sidebar_visible,
            Message::ToggleRightSidebar => self.right_sidebar_visible = !self.right_sidebar_visible,
            Message::LeftSidebarClicked(mode) => {
                if self.left_sidebar_visible && self.left_sidebar_mode == mode {
                    self.left_sidebar_visible = false;
                } else {
                    self.left_sidebar_visible = true;
                    self.left_sidebar_mode = mode;
                }
            }
            Message::RightSidebarClicked(mode) => {
                if self.right_sidebar_visible && self.right_sidebar_mode == mode {
                    self.right_sidebar_visible = false;
                } else {
                    self.right_sidebar_visible = true;
                    self.right_sidebar_mode = mode;
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut main_content = Row::new().height(Length::Fill);

        if self.left_sidebar_visible {
            main_content = main_content.push(sidebar_left(self));
        }

        main_content = main_content.push(center_stage(self));

        if self.right_sidebar_visible {
            main_content = main_content.push(sidebar_right(self));
        }

        column![main_content, control_panel(self),].into()
    }
}

fn main() -> iced::Result {
    run(App::update, App::view)
}
