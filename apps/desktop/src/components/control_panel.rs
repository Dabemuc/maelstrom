use crate::{App, LeftSidebarMode, Message, RightSidebarMode};
use iced::widget::tooltip::Position;
use iced::widget::{Space, button, container, row, svg, tooltip};
use iced::{Alignment, Element, Length};

pub fn control_panel(state: &App) -> Element<'_, Message> {
    let left_controls = row![
        icon_button(
            "assets/icons/folder.svg",
            "File Navigator",
            state.left_sidebar_visible && state.left_sidebar_mode == LeftSidebarMode::FileNavigator
        )
        .on_press(Message::LeftSidebarClicked(LeftSidebarMode::FileNavigator)),
        icon_button(
            "assets/icons/layers.svg",
            "Collections",
            state.left_sidebar_visible && state.left_sidebar_mode == LeftSidebarMode::Collections
        )
        .on_press(Message::LeftSidebarClicked(LeftSidebarMode::Collections)),
    ]
    .spacing(10);

    let right_controls = row![
        icon_button(
            "assets/icons/edit.svg",
            "Edit",
            state.right_sidebar_visible && state.right_sidebar_mode == RightSidebarMode::Edit
        )
        .on_press(Message::RightSidebarClicked(RightSidebarMode::Edit)),
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
    .padding(10)
    .width(Length::Fill)
    .into()
}

fn icon_button<'a>(
    svg_path: &'a str,
    label: &'a str,
    is_active: bool,
) -> iced::widget::Button<'a, Message> {
    let handle = svg::Handle::from_path(svg_path);

    let icon = svg(handle)
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0));

    let mut btn = button(tooltip(icon, label, Position::Top));

    if is_active {
        btn = btn.style(button::primary);
    } else {
        btn = btn.style(button::secondary);
    }

    btn
}
