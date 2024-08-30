use crate::{
    graphics::Context,
    layout::{Layout, MeasureNode, Rect, Size},
    styling::Style,
};
use skia_safe::{Data, Image};
use std::{fmt::Display, io::Cursor};

#[derive(Debug, Clone)]
pub struct ImageElement {
    pub data: Image,
}

impl ImageElement {
    /// # Panics
    ///
    /// Panic if can't create `Data` from `data` or `Image` from `Data`
    pub fn new<T: Display>(name: T, data: Vec<u8>) -> Self {
        let length = data.len();
        let data = Cursor::new(data);

        let data = Data::from_stream(data, length)
            .and_then(Image::from_encoded)
            .unwrap_or_else(|| panic!("failed to get {name} image data"));

        Self { data }
    }
}

impl Layout<Context> for ImageElement {
    fn measure(&self, _: &Context, style: &Style, parent: &Rect) -> MeasureNode {
        MeasureNode::new(
            style,
            parent,
            #[allow(clippy::cast_precision_loss)]
            Size::new(self.data.width() as f32, self.data.height() as f32),
        )
    }
}
