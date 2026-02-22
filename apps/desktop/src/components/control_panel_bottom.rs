use crate::{App, LeftSidebarMode, Message, RightSidebarMode};
use iced::widget::tooltip::Position;
use iced::widget::{Space, button, container, row, svg, tooltip};
use iced::{Alignment, Element, Length};

pub fn control_panel_bottom(state: &App) -> Element<'_, Message> {
    let left_controls = row![
        icon_button(
            svg::Handle::from_memory(include_bytes!("../../assets/icons/folder.svg")),
            "Navigator",
            state.left_sidebar_mode == LeftSidebarMode::Navigator
        )
        .on_press(Message::LeftSidebarClicked(LeftSidebarMode::Navigator)),
        icon_button(
            svg::Handle::from_memory(include_bytes!("../../assets/icons/layers.svg")),
            "Collections",
            state.left_sidebar_mode == LeftSidebarMode::Collections
        )
        .on_press(Message::LeftSidebarClicked(LeftSidebarMode::Collections)),
    ]
    .spacing(10);

    let right_controls = row![
        icon_button(
            svg::Handle::from_memory(include_bytes!("../../assets/icons/edit.svg")),
            "Develop",
            state.right_sidebar_mode == RightSidebarMode::Develop
        )
        .on_press(Message::RightSidebarClicked(RightSidebarMode::Develop)),
    ]
    .spacing(10);

    container(
        row![
            left_controls,
            Space::new().width(Length::Fill),
            right_controls,
        ]
        .align_y(Alignment::Center),
    )
    .padding(5)
    .width(Length::Fill)
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.strong.color.into()),
            text_color: Some(palette.background.strong.text),
            ..container::Style::default()
        }
    })
    .into()
}

fn icon_button<'a>(
    handle: svg::Handle,
    label: &'a str,
    is_active: bool,
) -> iced::widget::Button<'a, Message> {
    let icon = svg(handle)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .style(move |theme: &iced::Theme, _status| {
            let palette = theme.extended_palette();
            iced::widget::svg::Style {
                color: Some(if is_active {
                    palette.primary.base.color
                } else {
                    palette.secondary.strong.color
                }),
            }
        });

    let btn = button(tooltip(icon, label, Position::Top)).padding(5);

    btn.style(|theme: &iced::Theme, status: button::Status| {
        let mut style = button::text(theme, status);
        if status == button::Status::Hovered {
            style.background = Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2).into());
            style.border.radius = 4.0.into();
        }
        style
    })
}
