use crate::muzui::{
    layout::{BoxLayout, Layout, Layouts, MasonryLayout, MeasureNode, Rect},
    node::Node,
    style::Style,
    Context,
};

#[derive(Debug, Clone)]
pub struct ContainerElement {
    pub(in super::super) layout: Layouts<Node>,
}

impl ContainerElement {
    #[must_use]
    pub const fn vertical() -> Self {
        Self {
            layout: Layouts::Box(BoxLayout::vertical()),
        }
    }

    #[must_use]
    pub const fn horizontal() -> Self {
        Self {
            layout: Layouts::Box(BoxLayout::horizontal()),
        }
    }

    #[must_use]
    pub const fn masonry(item_width: f32) -> Self {
        Self {
            layout: Layouts::Masonry(MasonryLayout {
                spacing: 0.0,
                item_width,
                children: Vec::new(),
            }),
        }
    }

    #[must_use]
    pub fn children(&self) -> &[Node] {
        match &self.layout {
            Layouts::Box(layout) => &layout.children,
            Layouts::Masonry(layout) => &layout.children,
        }
    }

    pub fn children_mut(&mut self) -> &mut Vec<Node> {
        match &mut self.layout {
            Layouts::Box(layout) => &mut layout.children,
            Layouts::Masonry(layout) => &mut layout.children,
        }
    }
}

impl Layout<Node> for ContainerElement {
    fn measure(&self, context: &Context, style: &Style, parent: &Rect) -> MeasureNode {
        match &self.layout {
            Layouts::Box(layout) => layout.measure(context, style, parent),
            Layouts::Masonry(layout) => layout.measure(context, style, parent),
        }
    }
}
