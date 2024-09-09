use crate::{
    graphics::Context, layout::{Layout, MeasureNode, Measurer, Rect}, styling::Style, ContainerElement, Element, GridElement, ImageElement, Masonry, TextElement
};
pub use builder::NodeBuilder;
use skia_safe::{Canvas, Paint, RRect, Rect as SkRect};
use std::fmt::Display;

mod builder;

#[derive(Debug, Clone)]
pub struct Node {
    style: Style,
    element: Element,
}

impl Node {
    #[must_use]
    pub fn column() -> NodeBuilder {
        NodeBuilder::new(Element::Container(ContainerElement::column()))
    }

    #[must_use]
    pub fn row() -> NodeBuilder {
        NodeBuilder::new(Element::Container(ContainerElement::row()))
    }

    #[must_use]
    pub fn grid() -> NodeBuilder {
        NodeBuilder::new(Element::Grid(GridElement::default()))
    }

    #[must_use]
    pub fn masonry(item_width: f32) -> NodeBuilder {
        NodeBuilder::new(Element::Masonry(Masonry::new(item_width)))
    }

    pub fn text(data: impl Into<String>) -> NodeBuilder {
        NodeBuilder::new(Element::Text(TextElement::new(data.into())))
    }

    #[must_use]
    pub fn image<T: Display>(name: T, data: Vec<u8>) -> NodeBuilder {
        NodeBuilder::new(Element::Image(ImageElement::new(name, data)))
    }

    #[must_use]
    pub fn into_builder(self) -> NodeBuilder {
        let Self { style, element } = self;

        NodeBuilder { style, element }
    }

    pub(crate) fn draw(&self, canvas: &Canvas, context: &Context, node: MeasureNode) {
        let rect = SkRect::from_xywh(
            node.outer.origin.x,
            node.outer.origin.y,
            node.outer.size.width,
            node.outer.size.height,
        );

        let round_rect = RRect::new_rect_radii(
            rect,
            &[
                (self.style.corner_radius.left, self.style.corner_radius.left).into(),
                (self.style.corner_radius.top, self.style.corner_radius.top).into(),
                (
                    self.style.corner_radius.right,
                    self.style.corner_radius.right,
                )
                    .into(),
                (
                    self.style.corner_radius.bottom,
                    self.style.corner_radius.bottom,
                )
                    .into(),
            ],
        );

        canvas.save();

        canvas.clip_rrect(round_rect, None, Some(true));

        let mut background = Paint::default();

        if let Some(color) = self.style.background {
            background.set_color(skia_safe::Color::new(color.as_u32()));

            canvas.draw_rrect(round_rect, &background);
        }

        match &self.element {
            Element::Container(container) => {
                for (node, measure_node) in container.children.iter().zip(node.children) {
                    node.draw(canvas, context, measure_node);
                }
            }
            Element::Masonry(masonry) => {
                for (node, measure_node) in masonry.children.iter().zip(node.children) {
                    node.draw(canvas, context, measure_node);
                }
            }
            Element::Grid(grid) => {
                for (node, grid_node) in grid.children.iter().zip(node.children) {
                    node.draw(canvas, context, grid_node);
                }
            }
            Element::Image(image) => {
                canvas.draw_image_rect(&image.data, None, rect, &background);
            }
            Element::Text(text) => {
                let mut paragraph = context.create_paragraph(&self.style, &text.data);

                paragraph.layout(if node.inner.size.width == 0.0 {
                    context.bounds.width()
                } else {
                    node.inner.size.width
                });

                paragraph.paint(canvas, (node.inner.origin.x, node.inner.origin.y));
            }
        }

        canvas.restore();

        // DEBUG INFO

        // let mut paint = Paint::default();

        // paint.set_color(Argb::new(255, 255, 255, 0).as_color());

        // let font = context.get_font(&self.style);
        // let info = format!(
        //     "{}x{} at {}x{}",
        //     node.size.width, node.size.height, node.position.x, node.position.y
        // );

        // let bounds = font.measure_str(&info, None).1;

        // canvas.draw_str(
        //     &info,
        //     (
        //         node.position.x + (node.size.width / 2.0) - (bounds.width() / 2.0),
        //         node.position.y + (node.size.height / 2.0),
        //     ),
        //     &font,
        //     &paint,
        // );
    }
}

impl Measurer<Context> for Node {
    fn measure(&self, context: &Context, parent: &Rect) -> MeasureNode {
        match &self.element {
            Element::Container(container) => container.measure(context, &self.style, parent),
            Element::Masonry(masonry) => masonry.measure(context, &self.style, parent),
            Element::Grid(grid) => grid.measure(context, &self.style, parent),
            Element::Image(image) => image.measure(context, &self.style, parent),
            Element::Text(text) => text.measure(context, &self.style, parent),
        }
    }

    fn get_style(&self) -> &Style {
        &self.style
    }
}
