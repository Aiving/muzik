use self::{
    font::{FontFamily, FontSize, FontSlant, FontWeight},
    position::Position,
    thickness::Thickness,
};
use crate::muzui::layout::Length;
use material_colors::color::Argb;

pub mod colors;
pub mod font;
pub mod position;
pub mod thickness;

#[derive(Debug, Default, Clone)]
pub struct Style {
    pub margin: Thickness,
    pub padding: Thickness,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub background: Option<Argb>,
    pub color: Option<Argb>,
    pub font_family: FontFamily,
    pub font_weight: FontWeight,
    pub font_slant: FontSlant,
    pub font_size: FontSize,
    pub corner_radius: Thickness,
    pub position: Position,
    pub x: Option<f32>,
    pub y: Option<f32>,
}
