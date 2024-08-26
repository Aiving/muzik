use crate::muzui::{
    layout::{Layout, MeasureNode, Rect, Size},
    node::Node,
    style::Style,
    Context,
};

#[derive(Debug, Clone)]
pub struct TextElement {
    pub data: String,
}

impl TextElement {
    #[must_use]
    pub const fn new(data: String) -> Self {
        Self { data }
    }
}

impl Layout<Node> for TextElement {
    fn measure(&self, context: &Context, style: &Style, parent: &Rect) -> MeasureNode {
        let mut paragraph = context.create_paragraph(style, self);

        paragraph.layout(if parent.size.width == 0.0 {
            context.size.width
        } else {
            parent.size.width
        });

        MeasureNode::new(
            style,
            parent,
            Size::new(paragraph.longest_line() + 1.0, paragraph.height()),
            crate::muzui::layout::NodeType::Text,
        )
    }
}
