use crate::{App, Message, ViewMode};
use iced::alignment::Horizontal;
use iced::widget::image::Handle;
use iced::widget::{Space, button, column, container, image, responsive, row, scrollable, text};
use iced::{Alignment, Element, Length};
use previews::preview_generation::PREVIEW_FILE_TYPE;

pub fn center_stage(state: &App) -> Element<'_, Message> {
    let content = match state.view_mode {
        ViewMode::NoCatalog => no_catalog_view(),
        ViewMode::Library => library_view(state),
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
            text("Get started by Creating or Selecting an existing Catalog")
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
            button(text("Select Catalog").size(16))
                .on_press(Message::SelectCatalog)
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

const CELL_SIZE: f32 = 150.0; // box width/height
const SPACING: f32 = 10.0;

fn library_view(state: &App) -> Element<'_, Message> {
    let previews: Vec<_> = state.workspace_state.previews.iter().collect();
    let catalog = state.catalog.clone().unwrap();

    scrollable(responsive(move |size| {
        let available_width = size.width;

        let per_row = ((available_width + SPACING) / (CELL_SIZE + SPACING))
            .floor()
            .max(1.0) as usize;

        let mut col = column![].spacing(SPACING);

        for chunk in previews.chunks(per_row) {
            let mut r = row![].spacing(SPACING);

            for pv in chunk {
                let path = catalog.root().join(catalog.cache_dir()).join(format!(
                    "{}.{}",
                    pv.hash,
                    PREVIEW_FILE_TYPE.get_file_extension()
                ));

                let img = image(Handle::from_path(path))
                    .width(Length::Fixed(CELL_SIZE))
                    .height(Length::Fixed(CELL_SIZE));

                r = r.push(
                    container(img)
                        .width(Length::Fixed(CELL_SIZE))
                        .height(Length::Fixed(CELL_SIZE)),
                );
            }

            col = col.push(r);
        }

        col.into()
    }))
    .into()
}
