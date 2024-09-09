pub use self::{
    container::ContainerElement, grid::*, image::ImageElement, masonry::Masonry, text::TextElement,
};

mod container;
mod flex;
mod grid;
mod image;
mod masonry;
mod text;

#[derive(Debug, Clone)]
pub enum Element {
    Container(ContainerElement),
    Masonry(Masonry),
    Image(ImageElement),
    Grid(GridElement),
    Text(TextElement),
}
