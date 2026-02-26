use crate::components::common::svg_button::icon_button;
use crate::components::divider::divider;
use crate::{App, Message};
use iced::Alignment::Center;
use iced::alignment::Horizontal::Right;
use iced::border::Radius;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Element, Length};

use iced::widget::Scrollable;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftSidebarMode {
    Navigator,
    Collections,
    Hidden,
}

pub fn sidebar_left(state: &App) -> Element<'_, Message> {
    let content = row![
        container(match state.left_sidebar_mode {
            LeftSidebarMode::Navigator => navigator_view(state),
            LeftSidebarMode::Collections => collections_view(state),
            LeftSidebarMode::Hidden => text("Hidden").into(),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Center)
        .align_y(Center),
        divider(true) // vertical divider towards center stage
    ];

    container(content)
        .width(300)
        .height(Length::Fill)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: Some(palette.background.weak.text),
                ..container::Style::default()
            }
        })
        .into()
}

fn navigator_view(state: &App) -> Element<'_, Message> {
    if let Some(_catalog) = &state.catalog {
        // Build directory elements
        let dir_elements: Vec<Element<_>> = state
            .imported_dirs
            .iter()
            .map(|path| {
                // build file tree from path
                let tree = build_folder_tree(
                    path,
                    &state.navigator_state.expanded,
                    &state.navigator_state.selected,
                    &state.navigator_state.image_counts,
                    0,
                );

                column![tree, divider(false)].padding([5, 5]).into()
            })
            .collect();

        // Start column with header row and divider
        let mut col = column![
            row![
                container(text("Imported Folders").width(Length::Shrink))
                    .padding(10)
                    .align_y(Center)
                    .clip(true),
                Space::new().width(Length::Fill),
                row![
                    icon_button(
                        svg::Handle::from_memory(include_bytes!("../../assets/icons/collapse.svg")),
                        "Collapse all",
                        false
                    )
                    .on_press(Message::NavigatorCollapseAll),
                    icon_button(
                        svg::Handle::from_memory(include_bytes!("../../assets/icons/plus.svg")),
                        "Import new folder",
                        false
                    )
                    .on_press(Message::ImportDirectory)
                ]
            ]
            .align_y(Center)
            .width(300),
            divider(false)
        ]
        .width(Length::Shrink)
        .height(Length::Fill);

        // Push each directory element individually
        let mut tree_col = column![];
        for elem in dir_elements {
            tree_col = tree_col.push(elem);
        }

        col = col.push(Scrollable::new(tree_col));

        col.into()
    } else {
        text("No Catalog").into()
    }
}

fn build_folder_tree(
    path: &PathBuf,
    expanded: &HashSet<PathBuf>,
    selected: &Option<PathBuf>,
    image_counts: &HashMap<PathBuf, usize>,
    depth: usize,
) -> Element<'static, Message> {
    let indent = 20 * depth as u16;
    let is_expanded = expanded.contains(path);
    let is_selected = selected.as_ref() == Some(path);

    let mut col = column![];

    // Folder name
    let label = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let mut no_count_yet = false;
    let image_count = image_counts.get(path).copied().unwrap_or_else(|| {
        no_count_yet = true;
        0
    });

    // --- Expand / Collapse Icon (only clickable area for toggling) ---
    let icon = if is_expanded { "▼" } else { "▶" };

    let expand_button = button(
        container(
            text(icon)
                .size(14)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Center)
                .align_y(Center),
        )
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(24)
    .height(24)
    .padding(0)
    .style(|theme: &iced::Theme, status: button::Status| {
        let mut style = button::text(theme, status);
        if status == button::Status::Hovered {
            style.background = Some(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.1).into());
            style.border.radius = 4.0.into();
        }
        style
    })
    .on_press(Message::ToggleDirectory(path.clone()));

    // --- Selectable row body ---
    let row_content = row![
        expand_button, // 24px
        container(text(label).size(14).wrapping(text::Wrapping::None))
            .clip(true)
            .width(Length::Fill), // 64px
        // Space::new().width(10), // 10px
        text(if !no_count_yet {
            image_count.to_string()
        } else {
            "...".to_string()
        })
        .size(14)
        .align_x(Right)
        .width(24), // 24px
    ]
    .spacing(8)
    .height(Length::Fill)
    .width(235)
    .align_y(Center);

    let row = row![
        Space::new().width(Length::Fixed(indent as f32)),
        row_content,
        Space::new().width(15),
    ];

    // Make the entire row (except icon button) selectable
    let selectable_row = button(row)
        .height(32) // your desired row height
        .padding([0, 8]) // remove default button padding
        .on_press(Message::SelectDirectory(path.clone()))
        .style(folder_row_style(is_selected));
    // .width(Length::Shrink);

    col = col.push(selectable_row);

    // --- Render children folders only ---
    if is_expanded {
        if let Ok(read_dir) = fs::read_dir(path) {
            let mut entries: Vec<_> = read_dir.flatten().filter(|e| e.path().is_dir()).collect();

            entries.sort_by_key(|e| e.path());

            for entry in entries {
                let child_path = entry.path();
                col = col.push(build_folder_tree(
                    &child_path,
                    expanded,
                    selected,
                    image_counts,
                    depth + 1,
                ));
            }
        }
    }

    if depth == 0 {
        col = col.push(Space::new().height(10));
    }

    Scrollable::new(col)
        .direction(Direction::Horizontal(Scrollbar::new()))
        .into()
}

fn folder_row_style(
    is_selected: bool,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |theme, status| {
        use iced::widget::button;

        let mut style = button::text(theme, status);

        if is_selected {
            style.background = Some(theme.extended_palette().secondary.weak.color.into());
            style.text_color = theme.palette().background;
            style.border.radius = Radius::new(5);
        }

        style
    }
}

fn collections_view(state: &App) -> Element<'_, Message> {
    if state.catalog.is_some() {
        text("Catalog collections placeholder").into()
    } else {
        text("No Catalog").into()
    }
}
