use crate::app::App;
use crate::components::divider::divider;
use crate::message::Message;
use crate::state::ViewMode;
use iced::alignment::Horizontal::Right;
use iced::widget::{
    button, checkbox, column, container, responsive, row, slider, text, text_input, Scrollable,
    Space,
};
use iced::{Alignment, Element, Length};
use io::catalog::edit_graph::{EditNodeKind, NodeParameters, ParamType, ParamValue};
use time::format_description;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use time::PrimitiveDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightSidebarMode {
    Metadata,
    Operations,
    Hidden,
}

pub fn sidebar_right(state: &App) -> Element<'_, Message> {
    let main_content = match state.view_mode {
        ViewMode::Library => match state.right_sidebar_mode {
            RightSidebarMode::Metadata => metadata_view(state),
            _ => text(format!(
                "No view for view mode {:?} and right sidebar mode {:?}",
                state.view_mode, state.right_sidebar_mode
            ))
            .into(),
        },

        ViewMode::Develop => match state.right_sidebar_mode {
            RightSidebarMode::Operations => operations_view(state),
            _ => text(format!(
                "No view for view mode {:?} and right sidebar mode {:?}",
                state.view_mode, state.right_sidebar_mode
            ))
            .into(),
        },
        _ => text(format!("No view for view mode {:?}", state.view_mode)).into(),
    };

    let content = row![
        divider(true),
        column![
            Scrollable::new(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
            footer_controls(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
    ];

    container(content)
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

fn footer_controls(state: &App) -> Element<'_, Message> {
    if state.view_mode != ViewMode::Develop
        || state.right_sidebar_mode != RightSidebarMode::Operations
    {
        return Space::new().height(Length::Fixed(0.0)).into();
    }

    let can_save = state.catalog.is_some()
        && state.develop_state.is_some()
        && state.workspace_state.selected_preview_hash.is_some();

    let mut save_button = button(text("Save").size(12)).padding([6, 12]);
    if can_save {
        save_button = save_button.on_press(Message::DevelopSaveRequested);
    }

    let export_button = button(text("Export").size(12))
        .padding([6, 12])
        .on_press(Message::DevelopExportRequested);

    container(
        row![save_button, export_button]
            .spacing(8)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([12, 16])
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

fn operations_view(state: &App) -> Element<'_, Message> {
    let Some(develop_state) = state.develop_state.as_ref() else {
        return text("No Develop state").into();
    };

    let mut content = column![section_title("Operations")].spacing(8);

    for &kind in EditNodeKind::all() {
        let mut default_node = kind.default_node();
        let node = develop_state
            .edit_graph
            .nodes
            .iter()
            .find(|node| node.kind() == kind)
            .unwrap_or(&default_node);

        let mut node_column = column![section_title(kind.label())].spacing(8);

        for spec in node.param_specs() {
            let value = node.get_param(spec.name);

            match (&spec.ty, value) {
                (ParamType::Float { min, max, step }, ParamValue::Float(current)) => {
                    let name = spec.name.to_string();
                    let name_for_input = name.clone();
                    let name_for_slider = name.clone();
                    let key = crate::state::develop::ParamKey {
                        kind,
                        name: name.clone(),
                    };
                    let value_label = develop_state
                        .param_inputs
                        .get(&key)
                        .cloned()
                        .unwrap_or_else(|| format!("{:.2}", current));
                    let header = row![
                        text(spec.label).size(12),
                        Space::new().width(Length::Fill),
                        text_input("", &value_label)
                            .width(Length::Fixed(72.0))
                            .size(12)
                            .on_input(move |value| Message::DevelopParamInputChanged {
                                kind,
                                name: name_for_input.clone(),
                                value,
                            }),
                    ]
                    .width(Length::Fill)
                    .align_y(Alignment::Center);

                    let input = slider(*min..=*max, current, move |new_value| {
                        Message::DevelopParamChanged {
                            kind,
                            name: name_for_slider.clone(),
                            value: ParamValue::Float(new_value),
                        }
                    })
                    .step(*step);

                    node_column = node_column.push(column![header, input].spacing(6));
                }
                (ParamType::Int { min, max }, ParamValue::Int(current)) => {
                    let name = spec.name.to_string();
                    let name_for_input = name.clone();
                    let name_for_slider = name.clone();
                    let key = crate::state::develop::ParamKey {
                        kind,
                        name: name.clone(),
                    };
                    let value_label = develop_state
                        .param_inputs
                        .get(&key)
                        .cloned()
                        .unwrap_or_else(|| current.to_string());
                    let header = row![
                        text(spec.label).size(12),
                        Space::new().width(Length::Fill),
                        text_input("", &value_label)
                            .width(Length::Fixed(72.0))
                            .size(12)
                            .on_input(move |value| Message::DevelopParamInputChanged {
                                kind,
                                name: name_for_input.clone(),
                                value,
                            }),
                    ]
                    .width(Length::Fill)
                    .align_y(Alignment::Center);

                    let input = slider(*min..=*max, current, move |new_value| {
                        Message::DevelopParamChanged {
                            kind,
                            name: name_for_slider.clone(),
                            value: ParamValue::Int(new_value),
                        }
                    });

                    node_column = node_column.push(column![header, input].spacing(6));
                }
                (ParamType::Bool, ParamValue::Bool(current)) => {
                    let name = spec.name.to_string();
                    let input = checkbox(current)
                        .label(spec.label)
                        .on_toggle(move |new_value| Message::DevelopParamChanged {
                            kind,
                            name: name.clone(),
                            value: ParamValue::Bool(new_value),
                        });

                    node_column = node_column.push(input);
                }
                _ => {}
            }
        }

        content = content.push(container(node_column).width(Length::Fill).padding([12, 16]));
        content = content.push(divider(false));
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
}

fn metadata_view(state: &App) -> Element<'_, Message> {
    let Some(selected_hash) = state.workspace_state.selected_preview_hash.as_ref() else {
        return container(
            text("Select an image to view metadata")
                .size(12)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16)
        .into();
    };

    let Some(preview) = state
        .workspace_state
        .previews
        .get(selected_hash)
        .or_else(|| state.workspace_state.preview_cache.get(selected_hash))
    else {
        return container(text("Selected image not available").size(12))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(16)
            .into();
    };

    let path = &preview.original_image.path;
    let filename = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown file".to_string());
    let file_path = path.to_string_lossy().to_string();
    let file_type = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string());
    let resolution = match (preview.original_image.width, preview.original_image.height) {
        (Some(width), Some(height)) => format!("{}x{} px", width, height),
        _ => "N/A".to_string(),
    };
    let file_size = preview
        .original_image
        .file_size
        .map(format_bytes)
        .unwrap_or_else(|| "N/A".to_string());

    responsive(move |size| {
        let _pane_width = size.width.max(1.0);
        let filename = filename.clone();
        let file_path = file_path.clone();
        let file_type = file_type.clone();
        let resolution = resolution.clone();
        let file_size = file_size.clone();

        let general_rows = vec![
            ("File path", file_path),
            ("File type", file_type),
            ("Resolution", resolution),
            ("File size", file_size),
            (
                "Created",
                format_display_datetime_opt(preview.original_image.created_at.as_deref()),
            ),
            ("Hash", preview.original_image.hash.clone()),
        ];

        let mut general_column = column![section_title("General")].spacing(8);
        for (label, value) in general_rows {
            general_column = general_column.push(metadata_row(label.to_string(), value));
        }

        let mut metadata_column = column![section_title("Metadata")].spacing(8);
        if let Some(meta) = preview.original_image.meta.as_ref() {
            metadata_column = metadata_column
                .push(metadata_row(
                    "Capture date".to_string(),
                    format_display_datetime_opt(meta.capture_date.as_deref()),
                ))
                .push(metadata_row(
                    "ISO".to_string(),
                    format_optional_number(meta.iso),
                ))
                .push(metadata_row(
                    "Shutter speed".to_string(),
                    format_optional_string(meta.shutter_speed.clone()),
                ))
                .push(metadata_row(
                    "Aperture".to_string(),
                    format_optional_aperture(meta.aperture),
                ))
                .push(metadata_row(
                    "Focal length".to_string(),
                    format_optional_focal_length(meta.focal_length),
                ))
                .push(metadata_row(
                    "Camera make".to_string(),
                    format_optional_string(meta.camera_make.clone()),
                ))
                .push(metadata_row(
                    "Camera model".to_string(),
                    format_optional_string(meta.camera_model.clone()),
                ))
                .push(metadata_row(
                    "Lens model".to_string(),
                    format_optional_string(meta.lens_model.clone()),
                ))
                .push(metadata_row(
                    "GPS latitude".to_string(),
                    format_optional_gps(meta.gps_latitude),
                ))
                .push(metadata_row(
                    "GPS longitude".to_string(),
                    format_optional_gps(meta.gps_longitude),
                ))
                .push(metadata_row(
                    "Orientation".to_string(),
                    format_optional_number(meta.orientation),
                ));
        } else {
            metadata_column = metadata_column.push(
                container(text("No EXIF metadata available").size(12))
                    .padding([8, 4])
                    .width(Length::Fill),
            );
        }

        let content = column![
            container(
                text(filename)
                    .size(16)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::Word),
            )
            .width(Length::Fill)
            .padding([12, 16]),
            divider(false),
            container(general_column)
                .width(Length::Fill)
                .padding([12, 16]),
            divider(false),
            container(metadata_column)
                .width(Length::Fill)
                .padding([12, 16]),
            Space::new().height(Length::Fixed(12.0)),
        ]
        .width(Length::Fill)
        .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    })
    .into()
}

fn section_title(label: &str) -> Element<'static, Message> {
    container(
        text(label.to_ascii_uppercase())
            .size(10)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([0, 4])
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        container::Style {
            text_color: Some(palette.background.strong.text),
            ..container::Style::default()
        }
    })
    .into()
}

fn metadata_row(label: String, value: String) -> Element<'static, Message> {
    let row_content = row![
        container(text(label).size(12).wrapping(text::Wrapping::None)).width(Length::Shrink),
        Space::new().width(12),
        container(
            text(value)
                .size(12)
                .align_x(Right)
                .wrapping(text::Wrapping::WordOrGlyph)
                .width(Length::Fill),
        )
        .width(Length::Fill),
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .spacing(12);

    container(row_content)
        .width(Length::Fill)
        .padding([6, 0])
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            container::Style {
                text_color: Some(palette.background.base.text),
                ..container::Style::default()
            }
        })
        .into()
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

fn format_optional_string(value: Option<String>) -> String {
    value.unwrap_or_else(|| "N/A".to_string())
}

fn format_display_datetime_opt(value: Option<&str>) -> String {
    match value {
        Some(raw) => format_display_datetime(raw).unwrap_or_else(|| raw.to_string()),
        None => "N/A".to_string(),
    }
}

fn format_display_datetime(value: &str) -> Option<String> {
    let output_format =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").ok()?;

    if let Ok(dt) = OffsetDateTime::parse(value, &Rfc3339) {
        return dt.format(&output_format).ok();
    }

    let capture_format =
        format_description::parse("[year]:[month]:[day] [hour]:[minute]:[second]").ok()?;
    let dt = PrimitiveDateTime::parse(value, &capture_format).ok()?;
    dt.format(&output_format).ok()
}

fn format_optional_number<T: std::fmt::Display>(value: Option<T>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

fn format_optional_aperture(value: Option<f32>) -> String {
    value
        .map(|v| format!("f/{:.1}", v))
        .unwrap_or_else(|| "N/A".to_string())
}

fn format_optional_focal_length(value: Option<f32>) -> String {
    value
        .map(|v| format!("{:.1} mm", v))
        .unwrap_or_else(|| "N/A".to_string())
}

fn format_optional_gps(value: Option<f64>) -> String {
    value
        .map(|v| format!("{:.6}", v))
        .unwrap_or_else(|| "N/A".to_string())
}
