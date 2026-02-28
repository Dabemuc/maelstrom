use crate::app::App;
use crate::components::divider::divider;
use crate::message::Message;
use crate::state::ViewMode;
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
