pub fn maelstrom_theme() -> iced::Theme {
    let palette = iced::theme::Palette {
        background: iced::color!(0x1e1e24),
        text: iced::color!(0xb0b0b5),
        primary: iced::color!(0x4A90E2),
        success: iced::color!(0x4CAF50),
        warning: iced::color!(0xFFC107),
        danger: iced::color!(0xF44336),
    };

    iced::Theme::custom_with_fn("Maelstrom Dark", palette, |palette| {
        let mut extended = iced::theme::palette::Extended::generate(palette);

        extended.background.base.color = iced::color!(0x1d1e24);
        extended.background.weak.color = iced::color!(0x23252b);
        extended.background.strong.color = iced::color!(0x2a2d34);

        extended
    })
}
