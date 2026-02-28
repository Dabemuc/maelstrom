use crate::message::Message;
use iced::border::Radius;
use iced::widget::tooltip::Position;
use iced::widget::{Tooltip, container, text};
use iced::{Border, Element};

/// Ergonomic tooltip wrapper that works with any widget and string-like content.
pub fn styled_tooltip<'a, W, S>(
    content: W,
    tooltip_content: S,
    position: Position,
) -> Tooltip<'a, Message>
where
    W: Into<Element<'a, Message>>,
    S: Into<String>,
{
    Tooltip::new(
        content.into(),
        container(text(tooltip_content.into()))
            .padding(10)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style {
                    background: Some(palette.background.weakest.color.into()),
                    text_color: Some(palette.background.weak.text),
                    border: Border {
                        color: palette.secondary.base.color,
                        radius: Radius::new(4),
                        width: 1.0,
                    },
                    ..Default::default()
                }
            }),
        position,
    )
}
