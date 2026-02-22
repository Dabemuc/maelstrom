use crate::{App, Message};
use iced::widget::{column, container, text};
use iced::{Alignment, Element, Length};

pub fn center_stage(_state: &App) -> Element<'_, Message> {
    let content = column![text("Center Stage").size(40),]
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
