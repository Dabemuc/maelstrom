use crate::{App, Message, ViewMode};
use iced::alignment::Horizontal;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length};

pub fn center_stage(_state: &App) -> Element<'_, Message> {
    let content = match _state.view_mode {
        ViewMode::NoCatalog => no_catalog_view(),
        _ => Space::new().width(Length::Fill).height(Length::Fill).into(),
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.base.color.into()),
                text_color: Some(palette.background.base.text),
                ..container::Style::default()
            }
        })
        .into()
}

fn no_catalog_view() -> Element<'static, Message> {
    column![
        Space::new().width(Length::Fill).height(Length::Fill),
        row![
            Space::new().width(Length::FillPortion(1)),
            text("Get started by Creating or Importing a Catalog")
                .size(25)
                .width(Length::FillPortion(2))
                .align_x(Horizontal::Center),
            Space::new().width(Length::FillPortion(1))
        ],
        row![
            button(text("Create Catalog").size(16))
                .on_press(Message::CreateCatalog)
                .padding([12, 24])
                .style(iced::widget::button::primary),
            button(text("Import Catalog").size(16))
                .on_press(Message::ImportCatalog)
                .padding([12, 24])
                .style(iced::widget::button::primary),
        ]
        .spacing(20),
        Space::new().width(Length::Fill).height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .spacing(40)
    .into()
}
