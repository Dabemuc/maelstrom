use std::sync::Arc;

use iced::advanced::{layout, renderer, widget::Tree, Layout, Widget};
use iced::{Element, Length, Rectangle, Renderer, Size};
use iced_wgpu::primitive::Renderer as PrimitiveRenderer;

use maelstrom_image::linear_image::LinearImage;

use crate::business::gpu::LinearImagePrimitive;

pub struct LinearImageView {
    image: Arc<LinearImage>,
}

impl LinearImageView {
    pub fn new(image: Arc<LinearImage>) -> Self {
        Self { image }
    }
}

impl<Message> Widget<Message, iced::Theme, Renderer> for LinearImageView {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = Size::new(self.image.width as f32, self.image.height as f32);
        layout::Node::new(limits.resolve(Length::Shrink, Length::Shrink, size))
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

        renderer.draw_primitive(bounds, LinearImagePrimitive::new(Arc::clone(&self.image)));
    }
}

impl<'a, Message> From<LinearImageView> for Element<'a, Message> {
    fn from(widget: LinearImageView) -> Self {
        Element::new(widget)
    }
}
