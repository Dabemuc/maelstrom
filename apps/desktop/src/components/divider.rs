use iced::{Element, Length, widget::container};

use crate::message::Message;

// A helper function to create a 1px divider
pub fn divider(vertival: bool) -> Element<'static, Message> {
    container("")
        .width(if vertival {
            Length::Fixed(1.0)
        } else {
            Length::Fill
        })
        .height(if vertival {
            Length::Fill
        } else {
            Length::Fixed(1.0)
        })
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.strong.color.into()),
                ..container::Style::default()
            }
        })
        .into()
}
