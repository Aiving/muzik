use crate::{
    styling::{
        Color, FontFamily, FontSize, FontSlant, FontWeight, Length, Position, Style, Thickness,
    },
    Column, Element, GridLength, Node, Row,
};

pub struct NodeBuilder {
    pub(super) style: Style,
    pub(super) element: Element,
}

impl NodeBuilder {
    #[must_use]
    pub fn new(element: Element) -> Self {
        Self {
            style: Style::new(),
            element,
        }
    }

    #[must_use]
    pub fn child(mut self, node: Node) -> Self {
        match &mut self.element {
            Element::Container(element) => element.children.push(node),
            Element::Masonry(element) => element.children.push(node),
            Element::Grid(element) => element.children.push(node),
            _ => {}
        }

        self
    }

    #[must_use]
    pub fn children(mut self, nodes: Vec<Node>) -> Self {
        match &mut self.element {
            Element::Container(element) => element.children.extend(nodes),
            Element::Masonry(element) => element.children.extend(nodes),
            Element::Grid(element) => element.children.extend(nodes),
            _ => {}
        }

        self
    }

    #[must_use]
    pub fn rows<T: IntoIterator<Item = GridLength>>(mut self, rows: T) -> Self {
        if let Element::Grid(element) = &mut self.element {
            element
                .rows
                .extend(rows.into_iter().map(|height| Row { height }));
        }

        self
    }

    #[must_use]
    pub fn columns<T: IntoIterator<Item = GridLength>>(mut self, columns: T) -> Self {
        if let Element::Grid(element) = &mut self.element {
            element
                .columns
                .extend(columns.into_iter().map(|width| Column { width }));
        }

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
    pub fn spacing(mut self, value: f32) -> Self {
        match &mut self.element {
            Element::Container(container) => container.spacing = value,
            Element::Masonry(masonry) => masonry.spacing = value,
            Element::Grid(grid) => grid.spacing = value,
            _ => {}
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
    pub fn build(self) -> Node {
        Node {
            style: self.style,
            element: self.element,
        }
    }
}
