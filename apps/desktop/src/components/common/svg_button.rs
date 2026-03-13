use crate::components::common::styled_tooltip::styled_tooltip;
use crate::message::Message;
use iced::Length;
use iced::widget::tooltip::Position;
use iced::widget::{button, svg};

pub fn icon_button<'a>(
    handle: svg::Handle,
    label: &'a str,
    is_active: bool,
    rotation_radians: f32,
) -> iced::widget::Button<'a, Message> {
    let icon = svg(handle)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .rotation(rotation_radians)
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

    let btn = button(styled_tooltip(icon, label, Position::Top)).padding(5);

    btn.style(|theme: &iced::Theme, status: button::Status| {
        let mut style = button::text(theme, status);
        if status == button::Status::Hovered {
            style.background = Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2).into());
            style.border.radius = 4.0.into();
        }
        style
    })
}
