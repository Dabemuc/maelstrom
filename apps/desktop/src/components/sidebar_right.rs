use crate::{App, Message};
use iced::widget::{column, container, text};
use iced::{Element, Length};

pub fn sidebar_right(_state: &App) -> Element<'_, Message> {
    let content = column![text("Right Sidebar").size(24),].width(200);

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
