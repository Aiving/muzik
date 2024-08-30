pub use self::{
    container::ContainerElement, image::ImageElement, masonry::Masonry, text::TextElement,
};

mod container;
mod image;
mod masonry;
mod text;

#[derive(Debug, Clone)]
pub enum Element {
    Container(ContainerElement),
    Masonry(Masonry),
    Image(ImageElement),
    Text(TextElement),
}
