use crate::Message;
use iced::Element;
use iced::widget::tooltip::Position;
use iced::widget::{Tooltip, container, text};

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
        container(text(tooltip_content.into())),
        position,
    )
}
