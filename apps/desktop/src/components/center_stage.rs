use crate::{App, Message};
use iced::Element;
use iced::widget::{column, text};

pub fn center_stage(_state: &App) -> Element<'_, Message> {
    column![
        text("Center Stage").size(40),
        text("This expands to fill the middle!")
    ]
    .width(iced::Length::Fill)
    .into()
}
