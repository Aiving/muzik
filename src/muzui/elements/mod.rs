use self::{container::ContainerElement, image::ImageElement, text::TextElement};

pub mod container;
pub mod image;
pub mod text;

#[derive(Debug, Clone)]
pub enum Element {
    Container(ContainerElement),
    Image(ImageElement),
    Text(TextElement),
}
