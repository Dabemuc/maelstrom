use iced::widget::row;
use iced::{Element, run};

mod components;
use components::center_stage::center_stage;
use components::sidebar_left::sidebar_left;
use components::sidebar_right::sidebar_right;

#[derive(Default)]
struct App {
    // init state
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Placeholder,
}

impl App {
    fn update(&mut self, _: Message) {}

    fn view(&self) -> Element<'_, Message> {
        row![sidebar_left(self), center_stage(self), sidebar_right(self)].into()
    }
}

fn main() -> iced::Result {
    run(App::update, App::view)
}
