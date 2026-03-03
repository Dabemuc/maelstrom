use std::collections::HashMap;

use crate::app::App;
use crate::components::divider::divider;
use crate::message::Message;
use crate::state::{Preview, ViewMode};
use iced::widget::{Scrollable, container, row, text};
use iced::{Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightSidebarMode {
    Develop,
    Hidden,
}

pub fn sidebar_right(state: &App) -> Element<'_, Message> {
    let content = row![
        divider(true),
        Scrollable::new(match state.view_mode {
            ViewMode::Library => library_view(state),
            ViewMode::Develop => develop_views(state),
            _ => text("Select an image and enter develop mode to view develop options").into(),
        })
        .width(Length::Fill)
        .height(Length::Fill)
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

fn library_view(state: &App) -> Element<'_, Message> {
    text(format!(
        "{:#?}",
        state
            .workspace_state
            .previews
            .iter()
            .filter(|pv| pv.1.original_image.meta.is_some())
            .collect::<HashMap<&String, &Preview>>()
    ))
    .into()
}
