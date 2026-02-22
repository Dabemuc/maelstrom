use crate::{App, Message};
use iced::Element;
use iced::widget::{column, text};

pub fn sidebar_left(_state: &App) -> Element<'_, Message> {
    column![text("Left Sidebar").size(24),].width(200).into()
}
