use crate::{App, Message};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

pub fn center_stage(state: &App) -> Element<'_, Message> {
    let content = column![
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
    .spacing(40);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.base.color.into()),
                text_color: Some(palette.background.base.text),
                ..container::Style::default()
            }
        })
        .into()
}
