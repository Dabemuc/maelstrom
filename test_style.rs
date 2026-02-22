use iced::{Theme, Color};
use iced::widget::button;

fn style_test(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::text(theme, status);
    if status == button::Status::Hovered {
        style.background = Some(Color::from_rgba(0.0, 0.0, 0.0, 0.2).into());
        style.border.radius = 4.0.into();
    }
    style
}
