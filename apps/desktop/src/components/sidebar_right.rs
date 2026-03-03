use crate::app::App;
use crate::components::divider::divider;
use crate::message::Message;
use crate::state::ViewMode;
use iced::alignment::Horizontal::Right;
use iced::widget::{Scrollable, Space, column, container, responsive, row, text};
use iced::{Alignment, Element, Length};
use time::OffsetDateTime;
use time::PrimitiveDateTime;
use time::format_description;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightSidebarMode {
    Metadata,
    Operations,
    Hidden,
}

pub fn sidebar_right(state: &App) -> Element<'_, Message> {
    let content = row![
        divider(true),
        Scrollable::new(match state.view_mode {
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
        })
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

fn operations_view(_state: &App) -> Element<'_, Message> {
    text("Develop views placeholder").into()
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
