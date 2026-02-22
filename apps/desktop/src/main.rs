use iced::{Element, run};

mod components;
use components::center_stage::center_stage;
use components::sidebar_left::sidebar_left;
use components::sidebar_right::sidebar_right;

struct App {
    pub left_sidebar_visible: bool,
    pub right_sidebar_visible: bool,
}

// init state
impl Default for App {
    fn default() -> Self {
        Self {
            left_sidebar_visible: true,
            right_sidebar_visible: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ToggleLeftSidebar,
    ToggleRightSidebar,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleLeftSidebar => self.left_sidebar_visible = !self.left_sidebar_visible,
            Message::ToggleRightSidebar => self.right_sidebar_visible = !self.right_sidebar_visible,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut layout = iced::widget::Row::new();

        if self.left_sidebar_visible {
            layout = layout.push(sidebar_left(self));
        }

        layout = layout.push(center_stage(self));

        if self.right_sidebar_visible {
            layout = layout.push(sidebar_right(self));
        }

        layout.into()
    }
}

fn main() -> iced::Result {
    run(App::update, App::view)
}
