use crate::components::divider::divider;
use crate::{App, Message};
use iced::widget::{column, container, row, text};
use iced::{Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftSidebarMode {
    Navigator,
    Collections,
    Hidden,
}

pub fn sidebar_left(state: &App) -> Element<'_, Message> {
    let content = row![
        column![
            text("Left Sidebar").size(24),
            text(format!("{:?}", state.left_sidebar_mode))
        ]
        .width(Length::Fill),
        divider(true)
    ]
    .width(200);

    container(content)
        .width(200)
        .height(Length::Fill)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: Some(palette.background.weak.text),
                ..container::Style::default()
            }
        })
        .into()
}
