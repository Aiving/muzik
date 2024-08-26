use skia_safe::{Data, Image};
use std::io::Cursor;

use crate::muzui::{
    layout::{Layout, MeasureNode, Rect, Size},
    node::Node,
    style::Style,
    Context,
};

#[derive(Debug, Clone)]
pub struct ImageElement {
    pub data: Image,
}

impl ImageElement {
    /// # Panics
    ///
    /// Panic if can't create `Data` from `data` or `Image` from `Data`
    pub fn new(data: Vec<u8>, name: &str) -> Self {
        let length = data.len();
        let data = Cursor::new(data);

        let data = Data::from_stream(data, length)
            .and_then(Image::from_encoded)
            .unwrap_or_else(|| panic!("failed to get {name} image data"));

        Self { data }
    }
}

impl Layout<Node> for ImageElement {
    fn measure(&self, _: &Context, style: &Style, parent: &Rect) -> MeasureNode {
        MeasureNode::new(
            style,
            parent,
            Size::new(self.data.width() as f32, self.data.height() as f32),
            crate::muzui::layout::NodeType::Image,
        )
    }
}
