use std::collections::HashMap;

use crate::app::App;
use crate::components::divider::divider;
use crate::message::Message;
use crate::state::{Preview, ViewMode};
use iced::widget::{Scrollable, container, row, text};
use iced::{Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightSidebarMode {
    Metadata,
    Operations,
    Hidden,
}

pub fn sidebar_right(state: &App) -> Element<'_, Message> {
    let content = row![
        divider(true),
        Scrollable::new(match state.view_mode {
            ViewMode::Library => match state.right_sidebar_mode {
                RightSidebarMode::Metadata => metadata_view(state),
                _ => text(format!(
                    "No view for view mode {:?} and right sidebar mode {:?}",
                    state.view_mode, state.right_sidebar_mode
                ))
                .into(),
            },

            ViewMode::Develop => match state.right_sidebar_mode {
                RightSidebarMode::Operations => operations_view(state),
                _ => text(format!(
                    "No view for view mode {:?} and right sidebar mode {:?}",
                    state.view_mode, state.right_sidebar_mode
                ))
                .into(),
            },
            _ => text(format!("No view for view mode {:?}", state.view_mode)).into(),
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

fn operations_view(_state: &App) -> Element<'_, Message> {
    text("Develop views placeholder").into()
}

fn metadata_view(state: &App) -> Element<'_, Message> {
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
