use crate::{App, Message};
use iced::widget::{Space, container, row};
use iced::{Alignment, Element, Length};

pub fn control_panel_top(_state: &App) -> Element<'_, Message> {
    container(row![Space::new().width(Length::Fill).height(28),].align_y(Alignment::Center))
        .width(Length::Fill)
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
