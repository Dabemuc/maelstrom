use crate::ViewMode;
use crate::components::divider::divider;
use crate::{App, Message};
use iced::widget::{container, row, text};
use iced::{Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightSidebarMode {
    Develop,
    Hidden,
}

pub fn sidebar_right(state: &App) -> Element<'_, Message> {
    let content = row![
        divider(true),
        match state.view_mode {
            ViewMode::Develop => develop_views(state),
            _ => text("Select an image and enter develop mode to view develop options").into(),
        }
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

fn develop_views(_state: &App) -> Element<'_, Message> {
    text("Develop views placeholder").into()
}
