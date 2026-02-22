use crate::{App, Message};
use iced::{Element, Alignment, Length};
use iced::widget::{button, column, row, text};

pub fn center_stage(state: &App) -> Element<'_, Message> {
    column![
        text("Center Stage").size(40),
        row![
            button(if state.left_sidebar_visible {
                "Hide Left Sidebar"
            } else {
                "Show Left Sidebar"
            })
            .on_press(Message::ToggleLeftSidebar),
            button(if state.right_sidebar_visible {
                "Hide Right Sidebar"
            } else {
                "Show Right Sidebar"
            })
            .on_press(Message::ToggleRightSidebar),
        ]
        .spacing(20)
    ]
    .width(Length::Fill)
    .align_x(Alignment::Center)
    .spacing(40)
    .into()
}
