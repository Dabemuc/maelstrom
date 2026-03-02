use crate::app::App;
use crate::components::common::styled_tooltip::styled_tooltip;
use crate::components::divider::divider;
use crate::message::Message;
use crate::state::workspace::SortingOption;
use crate::state::{Preview, PreviewState, ViewMode};
use iced::Alignment::Center;
use iced::alignment::Horizontal;
use iced::widget::tooltip::Position;
use iced::widget::{
    Space, button, column, container, image, pick_list, responsive, row, scrollable, text,
};
use iced::{Alignment, Element, Length};

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
    // Create a vector of preview references ordered by the sorted keys
    let previews: Vec<(&String, &Preview)> = state
        .workspace_state
        .sorted_preview_keys
        .iter()
        .filter_map(|k| {
            state
                .workspace_state
                .previews
                .get(k)
                .map(|preview| (k, preview))
        })
        .collect();

    column![
        row![
            Space::new().width(Length::Fill),
            text("Sort by"),
            pick_list(
                vec![SortingOption::FileName],
                Some(&state.workspace_state.selected_sorting_option),
                Message::SortingOptionSelected
            )
            .placeholder(state.workspace_state.selected_sorting_option.to_string())
        ]
        .padding(10)
        .align_y(Center),
        divider(false),
        scrollable(responsive(move |size| {
            let available_width = size.width;

            let per_row = ((available_width + SPACING) / (CELL_SIZE + SPACING))
                .floor()
                .max(1.0) as usize;

            let mut col = column![].spacing(SPACING);

            for chunk in previews.chunks(per_row) {
                let mut r = row![].spacing(SPACING);

                for pv in chunk {
                    let img = image(
                        if pv.1.img_handle.is_some() && pv.1.preview_state == PreviewState::Ok {
                            pv.1.img_handle.clone().unwrap().clone()
                        } else {
                            state
                                .workspace_state
                                .handle_to_missing_preview_placeholder
                                .clone()
                        },
                    )
                    .width(Length::Fixed(CELL_SIZE))
                    .height(Length::Fixed(CELL_SIZE));

                    r = r.push(
                        container(styled_tooltip(
                            img,
                            pv.1.path_to_original.to_str().unwrap_or(""),
                            Position::Top,
                        ))
                        .width(Length::Fixed(CELL_SIZE))
                        .height(Length::Fixed(CELL_SIZE))
                        .padding(10),
                    );
                }

                col = col.push(r);
            }

            col.into()
        }))
    ]
    .into()
}
