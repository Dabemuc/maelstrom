use crate::{App, Message};
use iced::Element;
use iced::widget::{column, text};

pub fn sidebar_left(state: &App) -> Element<'_, Message> {
    column![
        text("Left Sidebar").size(24),
        text(format!("Value: {}", state.value))
    ]
    .width(200)
    .into()
}
