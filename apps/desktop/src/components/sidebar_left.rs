use crate::components::common::svg_button::icon_button;
use crate::components::divider::divider;
use crate::{App, Message};
use iced::alignment::Horizontal::Right;
use iced::border::Radius;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Alignment::Center, Element, Length};

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

/* ============================================================
   PUBLIC SIDEBAR
============================================================ */

pub fn sidebar_left(state: &App) -> Element<'_, Message> {
    let content = row![
        match state.left_sidebar_mode {
            LeftSidebarMode::Navigator => navigator_view(state),
            LeftSidebarMode::Collections => collections_view(state),
            LeftSidebarMode::Hidden => text("Hidden").into(),
        },
        divider(true)
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

/* ============================================================
   NAVIGATOR VIEW
============================================================ */

fn navigator_view(state: &App) -> Element<'_, Message> {
    if state.catalog.is_none() {
        return text("No Catalog").width(Length::Fill).into();
    }

    // -------- HEADER (spans full width) --------
    let header = row![
        Space::new().width(Length::Fill),
        icon_button(
            svg::Handle::from_memory(include_bytes!("../../assets/icons/plus.svg")),
            "Import new folder",
            false
        )
        .on_press(Message::ImportDirectory)
    ]
    .height(32)
    .align_y(Center);

    // -------- BUILD TREE ROWS --------
    let mut rows: Vec<TreeRow> = Vec::new();

    for path in &state.imported_dirs {
        rows.extend(build_folder_rows(
            path,
            &state.navigator_state.expanded,
            &state.navigator_state.selected,
            &state.navigator_state.image_counts,
            0,
        ));
    }

    let mut tree_column = column![];
    let mut count_column = column![];

    for row_data in rows {
        tree_column = tree_column.push(row_data.tree);
        count_column = count_column.push(row_data.count);
        if row_data.is_root {
            tree_column = tree_column.push(divider(false));
            count_column = count_column.push(divider(false));
        }
    }

    // Horizontal scroll ONLY on tree column
    let tree_scroll = Scrollable::new(tree_column)
        .direction(Direction::Horizontal(Scrollbar::new()))
        .width(Length::Fill);

    let body_row = row![
        tree_scroll,
        container(count_column).width(25),
        Space::new().width(15)
    ]
    .padding(12);

    let vertical_scroll = Scrollable::new(body_row).height(Length::Fill);

    column![header, divider(false), vertical_scroll]
        .height(Length::Fill)
        .into()
}

/* ============================================================
   TREE ROW STRUCT
============================================================ */

struct TreeRow {
    tree: Element<'static, Message>,
    count: Element<'static, Message>,
    is_root: bool,
}

/* ============================================================
   RECURSIVE TREE BUILDER
============================================================ */

fn build_folder_rows(
    path: &PathBuf,
    expanded: &HashSet<PathBuf>,
    selected: &Option<PathBuf>,
    image_counts: &HashMap<PathBuf, u32>,
    depth: usize,
) -> Vec<TreeRow> {
    let indent = 20.0 * depth as f32;
    let is_expanded = expanded.contains(path);
    let is_selected = selected.as_ref() == Some(path);

    let label = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let image_count = image_counts.get(path).copied().unwrap_or(0);

    let icon = if is_expanded { "▼" } else { "▶" };

    let expand_button = button(text(icon).size(14))
        .width(24)
        .height(24)
        .padding(0)
        .on_press(Message::ToggleDirectory(path.clone()));

    let tree_content = row![
        Space::new().width(indent),
        expand_button,
        container(text(label).size(14).wrapping(text::Wrapping::None)).width(Length::Shrink)
    ]
    .spacing(8)
    .align_y(Center);

    let selectable_row = button(tree_content)
        .height(32)
        .padding([0, 8])
        .on_press(Message::SelectDirectory(path.clone()))
        .style(folder_row_style(is_selected));

    let count_cell = container(text(image_count.to_string()).size(14).align_x(Right))
        .width(Length::Fill)
        .height(32)
        .align_x(Right)
        .align_y(Center);

    let mut rows = vec![TreeRow {
        tree: selectable_row.into(),
        count: count_cell.into(),
        is_root: depth == 0,
    }];

    if is_expanded {
        if let Ok(read_dir) = fs::read_dir(path) {
            let mut entries: Vec<_> = read_dir.flatten().filter(|e| e.path().is_dir()).collect();

            entries.sort_by_key(|e| e.path());

            for entry in entries {
                rows.extend(build_folder_rows(
                    &entry.path(),
                    expanded,
                    selected,
                    image_counts,
                    depth + 1,
                ));
            }
        }
    }

    rows
}

/* ============================================================
   ROW STYLE
============================================================ */

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

/* ============================================================
   COLLECTIONS VIEW
============================================================ */

fn collections_view(state: &App) -> Element<'_, Message> {
    if state.catalog.is_some() {
        text("Catalog collections placeholder").into()
    } else {
        text("No Catalog").into()
    }
}
