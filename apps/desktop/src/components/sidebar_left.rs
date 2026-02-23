use crate::components::common::svg_button::icon_button;
use crate::components::divider::divider;
use crate::{App, Message};
use iced::Alignment::Center;
use iced::widget::{Space, button, column, container, row, svg, text};
use iced::{Element, Length};

use std::collections::HashSet;
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
        .padding(10)
        .align_x(Center)
        .align_y(Center),
        divider(true)
    ];

    container(content)
        .width(200)
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
                let tree = build_file_tree(path, &state.navigator_state.expanded, 0);

                column![
                    text(path.to_string_lossy()),
                    divider(false),
                    tree,
                    divider(false)
                ]
                .into()
            })
            .collect();

        // Start column with header row and divider
        let mut col = column![
            row![
                Space::new().width(Length::Fill),
                icon_button(
                    svg::Handle::from_memory(include_bytes!("../../assets/icons/plus.svg")),
                    "Import new folder",
                    false
                )
                .on_press(Message::ImportDirectory)
            ]
            .align_y(Center),
            divider(false)
        ]
        .width(Length::Fill)
        .height(Length::Fill);

        // Push each directory element individually
        for elem in dir_elements {
            col = col.push(elem);
        }

        col.into()
    } else {
        text("No Catalog").into()
    }
}

fn build_file_tree(
    path: &PathBuf,
    expanded: &HashSet<PathBuf>,
    depth: usize,
) -> Element<'static, Message> {
    let indent = 20 * depth as u16;
    let is_expanded = expanded.contains(path);

    let mut col = column![];

    // Safe file/folder name extraction
    let label = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    // --- Render current node ---
    let node: Element<_> = if path.is_dir() {
        let icon = if is_expanded { "▼" } else { "▶" };

        row![
            Space::new().width(Length::Fixed(indent as f32)),
            button(text(format!("{} {}", icon, label)))
                .on_press(Message::ToggleDirectory(path.clone()))
        ]
        .into()
    } else {
        row![
            Space::new().width(Length::Fixed(indent as f32)),
            text(label).size(14)
        ]
        .into()
    };

    col = col.push(node);

    // --- Render children if expanded ---
    if is_expanded && path.is_dir() {
        if let Ok(read_dir) = fs::read_dir(path) {
            let mut entries: Vec<_> = read_dir.flatten().collect();

            // Sort: folders first, then files, alphabetically
            entries.sort_by_key(|e| {
                let p = e.path();
                (!p.is_dir(), p)
            });

            for entry in entries {
                let child_path = entry.path();
                col = col.push(build_file_tree(&child_path, expanded, depth + 1));
            }
        }
    }

    col.into()
}

fn collections_view(state: &App) -> Element<'_, Message> {
    if state.catalog.is_some() {
        text("Catalog collections placeholder").into()
    } else {
        text("No Catalog").into()
    }
}
