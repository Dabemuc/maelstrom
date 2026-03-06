use crate::app::App;
use crate::business::workspace::FolderNode;
use crate::components::common::svg_button::icon_button;
use crate::components::divider::divider;
use crate::message::Message;
use iced::alignment::Horizontal::Right;
use iced::border::Radius;
use iced::widget::scrollable::Scrollbar;
use iced::widget::{button, column, container, mouse_area, responsive, row, svg, text, Space};
use iced::Alignment::Center;
use iced::{Element, Length};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftSidebarMode {
    Directories,
    Collections,
    Hidden,
}

pub fn sidebar_left(state: &App) -> Element<'_, Message> {
    let content = row![
        container(match state.left_sidebar_mode {
            LeftSidebarMode::Directories => directories_view(state),
            LeftSidebarMode::Collections => collections_view(state),
            LeftSidebarMode::Hidden => text("Hidden").into(),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Center)
        .align_y(Center),
        divider(true)
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    container(content)
        .width(Length::Fill)
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

fn directories_view(state: &App) -> Element<'_, Message> {
    if state.catalog.is_none() {
        return text("No Catalog").into();
    }

    let roots: Vec<PathBuf> = if !state.imported_dirs.is_empty() {
        state.imported_dirs.clone()
    } else {
        state.workspace_state.model.root_folders.clone()
    };

    responsive(move |size| {
        let mut tree_col = column![];

        for root in roots.iter() {
            let is_scanning = state.workspace_state.roots_scanning.contains(root);

            tree_col = if is_scanning {
                tree_col
                    .push(build_loading_root_row(
                        root,
                        0,
                        &state.directories_state.context_menu_root,
                        state.directories_state.context_menu_open,
                        true,
                    ))
                    .push(divider(false))
            } else {
                tree_col
                    .push(build_folder_tree(
                        root,
                        &state.directories_state.expanded,
                        &state.directories_state.selected,
                        &state.workspace_state.model.folder_index,
                        &state.directories_state.context_menu_root,
                        state.directories_state.context_menu_open,
                        0,
                    ))
                    .push(divider(false))
            };
        }

        let pane_width = size.width.max(1.0);
        let tree_scroll = iced::widget::Scrollable::new(tree_col.width(Length::Fixed(pane_width)))
            .direction(iced::widget::scrollable::Direction::Vertical(
                Scrollbar::new(),
            ))
            .width(Length::Fixed(pane_width))
            .height(Length::Fill);

        let content = column![
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
                        false,
                        0.0
                    )
                    .on_press(Message::DirectoriesCollapseAll),
                    icon_button(
                        svg::Handle::from_memory(include_bytes!("../../assets/icons/plus.svg")),
                        "Import new folder",
                        false,
                        0.0
                    )
                    .on_press(Message::ImportDirectory)
                ]
            ]
            .align_y(Center)
            .width(Length::Fixed(pane_width)),
            divider(false),
            container(tree_scroll)
                .width(Length::Fixed(pane_width))
                .height(Length::Fill)
        ]
        .width(Length::Fixed(pane_width))
        .height(Length::Fill);

        container(content)
            .width(Length::Fixed(pane_width))
            .height(Length::Fill)
            .into()
    })
    .into()
}

fn build_loading_root_row(
    path: &PathBuf,
    depth: usize,
    context_menu_root: &Option<PathBuf>,
    context_menu_open: bool,
    is_scanning: bool,
) -> Element<'static, Message> {
    let indent = 20 * depth as u16;
    let label = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let loading_icon = container(
        text("▶")
            .size(14)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Center)
            .align_y(Center),
    )
    .width(24)
    .height(24);

    let row_content = row![
        loading_icon,
        container(
            text(format!("{label} (loading...)"))
                .size(14)
                .wrapping(text::Wrapping::None)
        )
        .clip(true)
        .width(Length::Fill),
        text("...").size(14).align_x(Right).width(24),
    ]
    .spacing(8)
    .height(Length::Fill)
    .width(Length::Fill)
    .align_y(Center);

    let root_row = container(
        row![
            Space::new().width(Length::Fixed(indent as f32)),
            row_content,
            Space::new().width(Length::Fixed(15.0)),
        ]
        .height(Length::Fill)
        .width(Length::Fill),
    )
    .height(32)
    .width(Length::Fill)
    .padding([0, 8])
    .style(|_theme: &iced::Theme| container::Style {
        text_color: Some(iced::Color::from_rgba8(140, 140, 140, 1.0)),
        ..container::Style::default()
    });

    let mut col = column![];

    col = col.push(mouse_area(root_row).on_right_press(Message::OpenRootContextMenu(path.clone())));

    if context_menu_open && context_menu_root.as_ref() == Some(path) {
        col = col.push(build_root_context_menu(path, is_scanning));
    }

    col.into()
}

fn build_folder_tree(
    path: &PathBuf,
    expanded: &HashSet<PathBuf>,
    selected: &Option<PathBuf>,
    folder_index: &HashMap<PathBuf, FolderNode>,
    context_menu_root: &Option<PathBuf>,
    context_menu_open: bool,
    depth: usize,
) -> Element<'static, Message> {
    let indent = 20 * depth as u16;
    let is_expanded = expanded.contains(path);
    let is_selected = selected.as_ref() == Some(path);
    let is_root = depth == 0;

    let mut col = column![];

    let label = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let node = folder_index.get(path);
    let image_count = node.map(|n| n.total_image_count).unwrap_or(0);
    let has_children = node.map(|n| !n.children.is_empty()).unwrap_or(false);

    let icon = if has_children {
        if is_expanded {
            "▼"
        } else {
            "▶"
        }
    } else {
        "•"
    };

    let expand_button = {
        let button = button(
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
        });

        if has_children {
            button.on_press(Message::ToggleDirectory(path.clone()))
        } else {
            button
        }
    };

    let row_content = row![
        expand_button,
        container(text(label).size(14).wrapping(text::Wrapping::None))
            .clip(true)
            .width(Length::Fill),
        text(image_count.to_string())
            .size(14)
            .align_x(Right)
            .width(24),
    ]
    .spacing(8)
    .height(Length::Fill)
    .width(Length::Fill)
    .align_y(Center);

    let row = row![
        Space::new().width(Length::Fixed(indent as f32)),
        row_content,
        Space::new().width(Length::Fixed(15.0)),
    ]
    .width(Length::Fill);

    let selectable_row = button(row)
        .height(32)
        .width(Length::Fill)
        .padding([0, 8])
        .on_press(Message::SelectDirectory(path.clone()))
        .style(folder_row_style(is_selected));

    if is_root {
        col = col.push(
            mouse_area(selectable_row).on_right_press(Message::OpenRootContextMenu(path.clone())),
        );
    } else {
        col = col.push(selectable_row);
    }

    if is_expanded {
        if let Some(node) = node {
            let mut children = node.children.clone();
            children.sort();

            for child_path in children {
                col = col.push(build_folder_tree(
                    &child_path,
                    expanded,
                    selected,
                    folder_index,
                    context_menu_root,
                    context_menu_open,
                    depth + 1,
                ));
            }
        }
    }

    if is_root && context_menu_open && context_menu_root.as_ref() == Some(path) {
        col = col.push(build_root_context_menu(path, false));
    }

    if depth == 0 {
        col = col.push(Space::new().height(10));
    }

    col.into()
}

fn build_root_context_menu(root: &PathBuf, is_scanning: bool) -> Element<'static, Message> {
    let refresh_button = {
        let button = button(
            text(if is_scanning {
                "Refresh (running...)"
            } else {
                "Refresh"
            })
            .size(13),
        )
        .padding([6, 10])
        .width(Length::Shrink);

        if is_scanning {
            button
        } else {
            button.on_press(Message::RefreshImportedRoot(root.clone()))
        }
    };

    container(
        row![
            Space::new().width(Length::Fixed(24.0)),
            row![
                refresh_button,
                button(text("Close").size(13))
                    .padding([6, 10])
                    .on_press(Message::CloseRootContextMenu)
            ]
            .spacing(6),
            Space::new().width(Length::Fill),
        ]
        .align_y(Center),
    )
    .padding([4, 8])
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.strong.color.into()),
            text_color: Some(palette.background.strong.text),
            ..container::Style::default()
        }
    })
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
