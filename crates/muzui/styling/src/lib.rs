#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub use self::{colors::Color, font::*, position::Position, thickness::Thickness};

mod colors;
mod font;
mod position;
mod thickness;

#[derive(Debug, Clone)]
pub enum Operation {
    Add(Length, Length),
    Sub(Length, Length),
    Mul(Length, Length),
    Div(Length, Length),
}

impl Operation {
    fn is_auto(&self) -> bool {
        match self {
            Self::Mul(left, right)
            | Self::Add(left, right)
            | Self::Sub(left, right)
            | Self::Div(left, right) => left.is_auto() || right.is_auto(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Length {
    Auto,
    Px(f32),
    Percent(f32),
    ParentWidth,
    ParentHeight,
    Dynamic(Vec<Operation>),
}

impl Length {
    #[must_use]
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto) || self.is_dynamic_and(|ops| ops.iter().any(Operation::is_auto))
    }

    fn is_dynamic_and<F: Fn(&[Operation]) -> bool>(&self, func: F) -> bool {
        if let Self::Dynamic(ops) = self {
            func(ops)
        } else {
            false
        }
    }
}

#[must_use]
pub const fn px(value: f32) -> Length {
    Length::Px(value)
}

#[must_use]
pub const fn percent(value: f32) -> Length {
    Length::Percent(value)
}

#[must_use]
pub const fn parent_width() -> Length {
    Length::ParentWidth
}

#[must_use]
pub const fn parent_height() -> Length {
    Length::ParentHeight
}

#[must_use]
pub const fn auto() -> Length {
    Length::Auto
}

pub fn dynamic<T: IntoIterator<Item = Operation>>(ops: T) -> Length {
    Length::Dynamic(ops.into_iter().collect())
}

#[derive(Debug, Default, Clone)]
pub struct Style {
    pub margin: Thickness,
    pub padding: Thickness,
    pub row: usize,
    pub row_span: usize,
    pub column: usize,
    pub column_span: usize,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub background: Option<Color>,
    pub color: Option<Color>,
    pub font_family: FontFamily,
    pub font_weight: FontWeight,
    pub font_slant: FontSlant,
    pub font_size: FontSize,
    pub corner_radius: Thickness,
    pub position: Position,
    pub x: Option<f32>,
    pub y: Option<f32>,
}

impl Style {
    #[must_use]
    pub fn new() -> Self {
        Self {
            row_span: 1,
            column_span: 1,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn builder() -> StyleBuilder {
        StyleBuilder { style: Self::new() }
    }
}

#[derive(Default)]
pub struct StyleBuilder {
    style: Style,
}

impl StyleBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn margin<T: Into<Thickness>>(mut self, value: T) -> Self {
        self.style.margin = value.into();

        self
    }

    #[must_use]
    pub fn padding<T: Into<Thickness>>(mut self, value: T) -> Self {
        self.style.padding = value.into();

        self
    }

    #[must_use]
    pub fn corner_radius<T: Into<Thickness>>(mut self, value: T) -> Self {
        self.style.corner_radius = value.into();

        self
    }

    #[must_use]
    pub const fn row(mut self, value: usize) -> Self {
        self.style.row = value;

        self
    }

    #[must_use]
    pub const fn row_span(mut self, value: usize) -> Self {
        self.style.row_span = value;

        self
    }

    #[must_use]
    pub const fn column(mut self, value: usize) -> Self {
        self.style.column = value;

        self
    }

    #[must_use]
    pub const fn column_span(mut self, value: usize) -> Self {
        self.style.column_span = value;

        self
    }

    #[must_use]
    pub fn background<T: Into<Color>>(mut self, value: T) -> Self {
        self.style.background = Some(value.into());

        self
    }

    #[must_use]
    pub fn color<T: Into<Color>>(mut self, value: T) -> Self {
        self.style.color = Some(value.into());

        self
    }

    #[must_use]
    pub fn font_family<T: Into<String>>(mut self, family: T) -> Self {
        self.style.font_family = FontFamily::new(family);

        self
    }

    #[must_use]
    pub fn font_size(mut self, size: f32) -> Self {
        self.style.font_size = FontSize::new(size);

        self
    }

    #[must_use]
    pub const fn font_weight(mut self, value: FontWeight) -> Self {
        self.style.font_weight = value;

        self
    }

    #[must_use]
    pub const fn font_slant(mut self, value: FontSlant) -> Self {
        self.style.font_slant = value;

        self
    }

    #[must_use]
    pub fn width(mut self, value: Length) -> Self {
        self.style.width = Some(value);

        self
    }

    #[must_use]
    pub fn height(mut self, value: Length) -> Self {
        self.style.height = Some(value);

        self
    }

    #[must_use]
    pub fn size(self, value: Length) -> Self {
        self.height(value.clone()).width(value)
    }

    #[must_use]
    pub const fn position(mut self, value: Position) -> Self {
        self.style.position = value;

        self
    }

    #[must_use]
    pub const fn x(mut self, value: f32) -> Self {
        self.style.x = Some(value);

        self
    }

    #[must_use]
    pub const fn y(mut self, value: f32) -> Self {
        self.style.y = Some(value);

        self
    }

    #[must_use]
    pub fn build(self) -> Style {
        self.style
    }
}
