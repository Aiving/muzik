// use super::Context;

use super::{
    language::{
        lexer::Token,
        parser::{Parse, Parser, Result},
    },
    style::Style,
    Context,
};

pub type Point = euclid::Point2D<f32, ()>;
pub type Rect = euclid::Rect<f32, ()>;
pub type Size = euclid::Size2D<f32, ()>;
pub type Vector2D = euclid::Vector2D<f32, ()>;

#[derive(Debug, Clone)]
pub enum Operation {
    Mul(Length, Length),
}

#[derive(Debug, Clone)]
pub enum Length {
    Px(f32),
    Percent(f32),
    Width,
    Dynamic(Vec<Operation>),
}

impl Length {
    fn eval(&self, available_space: f32, available: Size) -> f32 {
        match self {
            Self::Px(value) => *value,
            Self::Percent(value) => available_space * (*value / 100.0),
            Self::Width => available.width,
            Self::Dynamic(operations) => {
                operations.iter().fold(0.0, |_, operation| match operation {
                    Operation::Mul(a, b) => {
                        a.eval(available_space, available) * b.eval(available_space, available)
                    }
                })
            }
        }
    }
}

impl Parse for Length {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let value = parser.consume_map(Token::try_as_f32)?;

        Ok(if parser.try_consume(&Token::Percent) {
            Self::Percent(value)
        } else {
            Self::Px(value)
        })
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Container,
    Text,
    Image,
}

#[derive(Debug, Clone)]
pub struct MeasureNode {
    pub ty: NodeType,
    pub outer: Rect,
    pub inner: Rect,
    pub children: Vec<MeasureNode>,
}

impl MeasureNode {
    #[must_use]
    pub fn new(
        Style {
            margin,
            padding,
            width,
            height,
            position,
            x,
            y,
            ..
        }: &Style,
        parent: &Rect,
        size: Size,
        ty: NodeType,
    ) -> Self {
        let width = width
            .as_ref()
            .map(|width| width.eval(parent.size.width, parent.size));
        let height = height
            .as_ref()
            .map(|height| height.eval(parent.size.height, parent.size));

        println!("{parent:?}");

        let outer = Rect::new(
            if position.is_absolute() {
                Point::new(
                    x.unwrap_or_default() + margin.left,
                    y.unwrap_or_default() + margin.top,
                )
            } else {
                Point::new(parent.origin.x + margin.left, parent.origin.y + margin.top)
            },
            Size::new(width.unwrap_or(size.width), height.unwrap_or(size.height)),
        );

        let inner = Rect::new(
            Point::new(outer.origin.x + padding.left, outer.origin.y + padding.top),
            Size::new(
                outer.size.width - padding.left - padding.right,
                outer.size.height - padding.top - padding.bottom,
            ),
        );

        Self {
            outer,
            inner,
            children: Vec::new(),
            ty,
        }
    }

    pub fn set_y(&mut self, style: &Style, y: f32) {
        self.outer.origin.y = y + style.margin.top;
        self.inner.origin.y = y + style.padding.top;
    }

    pub fn set_x(&mut self, style: &Style, x: f32) {
        self.outer.origin.x = x + style.margin.left;
        self.inner.origin.x = x + style.padding.left;
    }

    pub fn set_height(&mut self, style: &Style, height: f32) {
        self.outer.size.height = height + style.padding.top + style.padding.bottom;
        self.inner.size.height = height;
    }

    pub fn set_width(&mut self, style: &Style, width: f32) {
        self.outer.size.width = width + style.padding.left + style.padding.right;
        self.inner.size.width = width;
    }

    #[must_use]
    pub fn offset(&self, point: Point) -> Rect {
        let mut inner = self.inner;

        inner.origin += point.to_vector();
        inner.size -= point.to_vector().to_size();

        inner
    }
}

pub(super) trait Measurer {
    fn measure(&self, context: &Context, parent: &Rect) -> MeasureNode;
    fn get_style(&self) -> &Style;
    fn get_ty(&self) -> NodeType;
}

pub(super) trait Layout<T> {
    fn measure(&self, context: &Context, style: &Style, parent: &Rect) -> MeasureNode;
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

impl Parse for Orientation {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume_map(|value| {
            value.try_as_ident().and_then(|ident| match ident {
                "vertical" => Some(Self::Vertical),
                "horizontal" => Some(Self::Horizontal),
                _ => None,
            })
        })
    }
}

#[derive(Debug, Clone)]
pub(super) struct BoxLayout<T: Measurer> {
    pub(super) orientation: Orientation,
    pub(super) spacing: f32,
    pub(super) children: Vec<T>,
}

impl<T: Measurer> BoxLayout<T> {
    pub(super) const fn new(orientation: Orientation) -> Self {
        Self {
            orientation,
            spacing: 0.0,
            children: Vec::new(),
        }
    }

    pub(super) const fn horizontal() -> Self {
        Self::new(Orientation::Horizontal)
    }

    pub(super) const fn vertical() -> Self {
        Self::new(Orientation::Vertical)
    }
}

impl<T: Measurer> Layout<T> for BoxLayout<T> {
    fn measure(&self, context: &Context, style: &Style, parent: &Rect) -> MeasureNode {
        let mut node = MeasureNode::new(style, parent, parent.size, NodeType::Container);

        let mut size = Size::zero();

        let count = self.children.len();

        match self.orientation {
            Orientation::Vertical => {
                for (index, child) in self.children.iter().enumerate() {
                    let style = child.get_style();

                    if style.position.is_relative() && index > 0 && index <= count {
                        size.height += self.spacing;
                    }

                    let child = child.measure(context, &node.offset(Point::new(0.0, size.height)));

                    if style.position.is_relative() {
                        size.height += child.outer.size.height;

                        if child.outer.size.width > size.width {
                            size.width = child.outer.size.width;
                        }
                    }

                    node.children.push(child);
                }
            }
            Orientation::Horizontal => {
                for (index, child) in self.children.iter().enumerate() {
                    let style = child.get_style();

                    if style.position.is_relative() && index > 0 && index <= count {
                        size.width += self.spacing;
                    }

                    let child = child.measure(context, &node.offset(Point::new(size.width, 0.0)));

                    if style.position.is_relative() {
                        size.width += child.outer.size.width;

                        if child.outer.size.height > size.height {
                            size.height = child.outer.size.height;
                        }
                    }

                    node.children.push(child);
                }
            }
        }

        // println!("children: {:#?}", node.children);

        if style.width.is_none() {
            node.set_width(style, size.width);
        }

        if style.height.is_none() {
            node.set_height(style, size.height);
        }

        node
    }
}

#[derive(Debug, Clone)]
pub(super) struct MasonryLayout<T: Measurer> {
    pub(super) spacing: f32,
    pub(super) item_width: f32,
    pub(super) children: Vec<T>,
}

impl<T: Measurer> MasonryLayout<T> {
    fn columns(&self, available_width: f32) -> f32 {
        (available_width / (self.item_width + self.spacing)).floor()
    }

    fn item_width(&self, available_width: f32) -> f32 {
        let columns = self.columns(available_width);

        self.spacing.mul_add(-(columns - 1.0), available_width) / columns
    }
}

fn create_base_node(
    Style {
        margin,
        padding,
        width,
        height,
        ..
    }: &Style,
    parent: &Rect,
    ty: NodeType,
) -> MeasureNode {
    let width = width
        .as_ref()
        .map(|width| width.eval(parent.size.width, parent.size));
    let height = height
        .as_ref()
        .map(|height| height.eval(parent.size.height, parent.size));

    let outer = Rect::new(
        Point::new(parent.origin.x + margin.left, parent.origin.y + margin.top),
        Size::new(
            width.unwrap_or(parent.size.width - margin.right),
            height.unwrap_or(parent.size.height - margin.bottom),
        ),
    );

    let inner = Rect::new(
        Point::new(outer.origin.x + padding.left, outer.origin.y + padding.top),
        Size::new(
            outer.size.width - padding.left - padding.right,
            outer.size.height - padding.top - padding.bottom,
        ),
    );

    MeasureNode {
        ty,
        outer,
        inner,
        children: Vec::new(),
    }
}

impl<T: Measurer> Layout<T> for MasonryLayout<T> {
    fn measure(&self, context: &Context, style: &Style, parent: &Rect) -> MeasureNode {
        let mut node = create_base_node(style, parent, NodeType::Container);

        let columns = self.columns(node.inner.size.width) as usize;
        let item_width = self.item_width(node.inner.size.width);

        for child in self
            .children
            .chunks(columns)
            .fold(Vec::<Vec<MeasureNode>>::new(), |mut rows, row| {
                let last = rows.last();

                let row = row
                    .iter()
                    .enumerate()
                    .map(|(x, child)| {
                        let y = if rows.is_empty() {
                            node.inner.origin.y
                        } else {
                            0.0
                        } + last
                            .and_then(|last| last.get(x))
                            .map(|last| last.outer.origin.y + last.outer.size.height + self.spacing)
                            .unwrap_or_default();
                        let x = (item_width + self.spacing).mul_add(x as f32, node.inner.origin.x);

                        let style = child.get_style();
                        let mut base = create_base_node(
                            style,
                            &Rect::new(Point::new(x, y), Size::new(item_width, 0.0)),
                            child.get_ty(),
                        );

                        let child = child.measure(context, &base.inner);

                        base.set_height(style, child.outer.size.height);
                        base.children = child.children;

                        base
                    })
                    .collect::<Vec<_>>();

                rows.push(row);

                rows
            })
            .into_iter()
            .flatten()
        {
            node.children.push(child);
        }

        node
    }
}

#[derive(Debug, Clone)]
pub(super) enum Layouts<T: Measurer> {
    Box(BoxLayout<T>),
    Masonry(MasonryLayout<T>),
}
