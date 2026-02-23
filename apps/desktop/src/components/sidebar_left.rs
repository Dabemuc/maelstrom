use crate::components::common::svg_button::icon_button;
use crate::components::divider::divider;
use crate::{App, Message};
use iced::Alignment::Center;
use iced::widget::{Space, column, container, row, svg, text};
use iced::{Element, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftSidebarMode {
    Navigator,
    Collections,
    Hidden,
}

pub fn sidebar_left(state: &App) -> Element<'_, Message> {
    let content = row![
        container(match state.left_sidebar_mode {
            LeftSidebarMode::Navigator => navigator_view(state),
            LeftSidebarMode::Collections => collections_view(state),
            LeftSidebarMode::Hidden => text("Hidden").into(),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .align_x(Center)
        .align_y(Center),
        divider(true)
    ];

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

fn navigator_view(state: &App) -> Element<'_, Message> {
    if state.catalog.is_some() {
        column![
            row![
                Space::new().width(Length::Fill),
                icon_button(
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/plus.svg")),
                    "Import new folder",
                    false
                )
                .on_press(Message::ImportDirectory)
            ]
            .align_y(Center),
            divider(false)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        text("No Catalog").into()
    }
}

fn collections_view(state: &App) -> Element<'_, Message> {
    if state.catalog.is_some() {
        text("Catalog collections placeholder").into()
    } else {
        text("No Catalog").into()
    }
}
