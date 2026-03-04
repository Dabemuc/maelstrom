use crate::app::App;
use crate::components::common::svg_button::icon_button;
use crate::components::sidebar_left::LeftSidebarMode;
use crate::components::sidebar_right::RightSidebarMode;
use crate::message::Message;
use crate::state::ViewMode;
use iced::widget::{Space, button, container, row, svg, text};
use iced::{Alignment, Element, Length};

pub fn control_panel_bottom(state: &App) -> Element<'_, Message> {
    let left_controls = row![
        icon_button(
            svg::Handle::from_memory(include_bytes!("../../assets/icons/folder.svg")),
            "Navigator",
            state.left_sidebar_mode == LeftSidebarMode::Navigator,
            0.0
        )
        .on_press(Message::LeftSidebarClicked(LeftSidebarMode::Navigator)),
        icon_button(
            svg::Handle::from_memory(include_bytes!("../../assets/icons/layers.svg")),
            "Collections",
            state.left_sidebar_mode == LeftSidebarMode::Collections,
            0.0
        )
        .on_press(Message::LeftSidebarClicked(LeftSidebarMode::Collections)),
    ]
    .spacing(10);

    let mut library_button = button(text("Library").size(14)).padding([6, 10]).style(
        |theme: &iced::Theme, status: iced::widget::button::Status| {
            let mut style = iced::widget::button::text(theme, status);
            if status == iced::widget::button::Status::Hovered {
                style.background = Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2).into());
                style.border.radius = 4.0.into();
            }
            style
        },
    );

    if state.view_mode != ViewMode::Library {
        library_button = library_button.on_press(Message::ViewModeSelected(ViewMode::Library));
    }

    let mut develop_button = button(text("Develop").size(14)).padding([6, 10]).style(
        |theme: &iced::Theme, status: iced::widget::button::Status| {
            let mut style = iced::widget::button::text(theme, status);
            if status == iced::widget::button::Status::Hovered {
                style.background = Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2).into());
                style.border.radius = 4.0.into();
            }
            style
        },
    );

    if state.workspace_state.selected_preview_hash.is_some() && state.view_mode != ViewMode::Develop
    {
        develop_button = develop_button.on_press(Message::ViewModeSelected(ViewMode::Develop));
    }

    let center_controls = row![library_button, develop_button].spacing(8);

    let right_controls = row![
        icon_button(
            svg::Handle::from_memory(include_bytes!("../../assets/icons/metadata.svg")),
            "Metadata",
            state.right_sidebar_mode == RightSidebarMode::Metadata,
            0.0
        )
        .on_press(Message::RightSidebarClicked(RightSidebarMode::Metadata)),
        icon_button(
            svg::Handle::from_memory(include_bytes!("../../assets/icons/edit.svg")),
            "Develop",
            state.right_sidebar_mode == RightSidebarMode::Operations,
            0.0
        )
        .on_press(Message::RightSidebarClicked(RightSidebarMode::Operations)),
    ]
    .spacing(10);

    container(
        row![
            left_controls,
            Space::new().width(Length::Fill),
            center_controls,
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
