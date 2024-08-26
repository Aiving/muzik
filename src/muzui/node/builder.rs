use super::Node;
use crate::muzui::{
    elements::Element,
    layout::{Layouts, Length, Operation},
    style::{
        font::{FontFamily, FontSize, FontSlant, FontWeight},
        position::Position,
        thickness::Thickness,
        Style,
    },
};
use material_colors::color::Argb;

pub struct NodeBuilder {
    style: Style,
    element: Element,
}

impl NodeBuilder {
    #[must_use]
    pub fn new(element: Element) -> Self {
        Self {
            style: Style::default(),
            element,
        }
    }

    #[must_use]
    pub fn child(mut self, node: Node) -> Self {
        if let Element::Container(container) = &mut self.element {
            container.children_mut().push(node);
        }

        self
    }

    #[must_use]
    pub fn children(mut self, nodes: Vec<Node>) -> Self {
        if let Element::Container(container) = &mut self.element {
            container.children_mut().extend(nodes);
        }

        self
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
    pub const fn background(mut self, value: Argb) -> Self {
        self.style.background = Some(value);

        self
    }

    #[must_use]
    pub const fn color(mut self, value: Argb) -> Self {
        self.style.color = Some(value);

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
    pub fn width(mut self, value: f32) -> Self {
        self.style.width = Some(Length::Px(value));

        self
    }

    #[must_use]
    pub fn dynamic_height(mut self, value: Vec<Operation>) -> Self {
        self.style.height = Some(Length::Dynamic(value));

        self
    }

    #[must_use]
    pub fn height(mut self, value: f32) -> Self {
        self.style.height = Some(Length::Px(value));

        self
    }

    #[must_use]
    pub fn size(self, value: f32) -> Self {
        self.height(value).width(value)
    }

    #[must_use]
    pub fn width_p(mut self, value: f32) -> Self {
        self.style.width = Some(Length::Percent(value));

        self
    }

    #[must_use]
    pub fn height_p(mut self, value: f32) -> Self {
        self.style.height = Some(Length::Percent(value));

        self
    }

    #[must_use]
    pub fn size_p(self, value: f32) -> Self {
        self.height_p(value).width_p(value)
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
    pub fn spacing(mut self, value: f32) -> Self {
        if let Element::Container(container) = &mut self.element {
            match &mut container.layout {
                Layouts::Box(layout) => layout.spacing = value,
                Layouts::Masonry(layout) => layout.spacing = value,
            }
        }

        self
    }

    #[must_use]
    pub fn build(self) -> Node {
        Node {
            style: self.style,
            element: self.element,
        }
    }
}
