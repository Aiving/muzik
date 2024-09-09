#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use muzui_styling::{Length, Operation, Style};

pub type Point = muzui_geometry::Point<f32>;
pub type Rect = muzui_geometry::Rect<f32>;
pub type Size = muzui_geometry::Size<f32>;

fn eval_length(length: &Length, available_space: f32, available: Size) -> f32 {
    match length {
        Length::Auto => 0.0,
        Length::Px(value) => *value,
        Length::Percent(value) => available_space * (*value / 100.0),
        Length::ParentWidth => available.width,
        Length::ParentHeight => available.height,
        Length::Dynamic(operations) => {
            operations.iter().fold(0.0, |_, operation| match operation {
                Operation::Add(a, b) => {
                    eval_length(a, available_space, available)
                        + eval_length(b, available_space, available)
                }
                Operation::Sub(a, b) => {
                    eval_length(a, available_space, available)
                        - eval_length(b, available_space, available)
                }
                Operation::Mul(a, b) => {
                    eval_length(a, available_space, available)
                        * eval_length(b, available_space, available)
                }
                Operation::Div(a, b) => {
                    eval_length(a, available_space, available)
                        / eval_length(b, available_space, available)
                }
            })
        }
    }
}

// impl Parse for Length {
//     fn parse(parser: &mut Parser) -> Result<Self> {
//         let value = parser.consume_map(Token::try_as_f32)?;

//         Ok(if parser.try_consume(&Token::Percent) {
//             Self::Percent(value)
//         } else {
//             Self::Px(value)
//         })
//     }
// }

#[derive(Debug, Clone)]
pub struct MeasureNode {
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
    ) -> Self {
        let width = width
            .as_ref()
            .map(|width| eval_length(width, parent.size.width, parent.size));
        let height = height
            .as_ref()
            .map(|height| eval_length(height, parent.size.height, parent.size));

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
        }
    }

    #[must_use]
    pub fn from_parent(
        Style {
            margin,
            padding,
            width,
            height,
            ..
        }: &Style,
        parent: &Rect,
    ) -> Self {
        let width = width
            .as_ref()
            .map(|width| eval_length(width, parent.size.width, parent.size));
        let height = height
            .as_ref()
            .map(|height| eval_length(height, parent.size.height, parent.size));

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

        Self {
            outer,
            inner,
            children: Vec::new(),
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

        inner += point;
        inner -= Size::from(point);

        inner
    }
}

pub trait Measurer<Context> {
    fn measure(&self, context: &Context, parent: &Rect) -> MeasureNode;
    fn get_style(&self) -> &Style;
}

pub trait Layout<Context> {
    fn measure(&self, context: &Context, style: &Style, parent: &Rect) -> MeasureNode;
}

// impl Parse for Orientation {
//     fn parse(parser: &mut Parser) -> Result<Self> {
//         parser.consume_map(|value| {
//             value.try_as_ident().and_then(|ident| match ident {
//                 "vertical" => Some(Self::Vertical),
//                 "horizontal" => Some(Self::Horizontal),
//                 _ => None,
//             })
//         })
//     }
// }
