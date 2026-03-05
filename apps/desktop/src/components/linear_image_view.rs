use std::sync::Arc;

use iced::advanced::{layout, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Renderer, Size};
use iced_wgpu::primitive::Renderer as PrimitiveRenderer;

use maelstrom_image::linear_image::LinearImage;

use crate::business::gpu::LinearImagePrimitive;
use crate::message::Message;
use crate::state::develop::ZoomMode;

#[derive(Debug, Default)]
struct LinearImageViewState {
    initialized: bool,
    last_fit_request: u64,
    dragging: bool,
    last_cursor: Option<iced::Point>,
}

pub struct LinearImageView {
    image: Arc<LinearImage>,
    zoom: f32,
    zoom_mode: ZoomMode,
    pan: [f32; 2],
    fit_request: u64,
    pan_enabled: bool,
}

impl LinearImageView {
    pub fn new(
        image: Arc<LinearImage>,
        zoom: f32,
        zoom_mode: ZoomMode,
        pan: [f32; 2],
        fit_request: u64,
        pan_enabled: bool,
    ) -> Self {
        Self {
            image,
            zoom,
            zoom_mode,
            pan,
            fit_request,
            pan_enabled,
        }
    }
}

impl Widget<Message, iced::Theme, Renderer> for LinearImageView {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = Size::new(self.image.width as f32, self.image.height as f32);
        layout::Node::new(limits.resolve(Length::Fill, Length::Fill, size))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        renderer.draw_primitive(
            bounds,
            LinearImagePrimitive::new(Arc::clone(&self.image), self.zoom, self.zoom_mode, self.pan),
        );
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(LinearImageViewState::default())
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<LinearImageViewState>();
        let needs_initial_fit = self.zoom_mode == ZoomMode::FitOnce && !state.initialized;
        let fit_requested = state.last_fit_request != self.fit_request;

        if needs_initial_fit || fit_requested {
            if needs_initial_fit {
                state.initialized = true;
            }
            if fit_requested {
                state.last_fit_request = self.fit_request;
            }

            let fit_zoom = compute_fit_zoom(layout.bounds(), &self.image);
            shell.publish(Message::DevelopZoomSet(fit_zoom));
        }

        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            if cursor.is_over(layout.bounds()) {
                let lines = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 50.0,
                };

                if lines.abs() > 0.0 {
                    let factor = 1.1_f32.powf(lines);
                    if let Some(position) = cursor.position_over(layout.bounds()) {
                        let (zoom, pan) = zoom_towards_point(
                            &self.image,
                            self.zoom,
                            self.pan,
                            layout.bounds(),
                            position,
                            factor,
                        );
                        shell.publish(Message::DevelopZoomSetPan { zoom, pan });
                    } else {
                        shell.publish(Message::DevelopZoomBy(factor));
                    }
                }
            }
        }

        if self.pan_enabled {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if let Some(position) = cursor.position_over(layout.bounds()) {
                        state.dragging = true;
                        state.last_cursor = Some(position);
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    state.dragging = false;
                    state.last_cursor = None;
                }
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    if state.dragging {
                        if let Some(last) = state.last_cursor {
                            let delta = [position.x - last.x, position.y - last.y];
                            state.last_cursor = Some(*position);

                            let pan_delta = [delta[0] / self.zoom, delta[1] / self.zoom];
                            shell.publish(Message::DevelopPanBy { delta: pan_delta });
                        } else {
                            state.last_cursor = Some(*position);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if !self.pan_enabled {
            return mouse::Interaction::None;
        }

        if cursor.is_over(layout.bounds()) {
            let dragging = tree.state.downcast_ref::<LinearImageViewState>().dragging;
            if dragging {
                mouse::Interaction::Grabbing
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a> From<LinearImageView> for Element<'a, Message> {
    fn from(widget: LinearImageView) -> Self {
        Element::new(widget)
    }
}

fn compute_fit_zoom(bounds: Rectangle, image: &LinearImage) -> f32 {
    let image_width = image.width as f32;
    let image_height = image.height as f32;
    if bounds.width > 0.0 && bounds.height > 0.0 {
        let scale_x = bounds.width / image_width;
        let scale_y = bounds.height / image_height;
        scale_x.min(scale_y)
    } else {
        1.0
    }
}

fn zoom_towards_point(
    image: &LinearImage,
    zoom: f32,
    pan: [f32; 2],
    bounds: Rectangle,
    cursor: iced::Point,
    factor: f32,
) -> (f32, [f32; 2]) {
    let new_zoom = (zoom * factor).clamp(0.05, 64.0);
    let image_center = [image.width as f32 * 0.5, image.height as f32 * 0.5];
    let center = [
        bounds.x + bounds.width * 0.5,
        bounds.y + bounds.height * 0.5,
    ];
    let cursor_offset = [cursor.x - center[0], cursor.y - center[1]];

    let image_point = [
        image_center[0] + (cursor_offset[0] - pan[0] * zoom) / zoom,
        image_center[1] + (cursor_offset[1] - pan[1] * zoom) / zoom,
    ];

    let new_pan = [
        (cursor_offset[0] / new_zoom) - (image_point[0] - image_center[0]),
        (cursor_offset[1] / new_zoom) - (image_point[1] - image_center[1]),
    ];

    (new_zoom, new_pan)
}
