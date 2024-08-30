use crate::{
    graphics::Context,
    layout::{Layout, MeasureNode, Measurer, Point, Rect, Size},
    styling::Style,
    Node,
};

#[derive(Debug, Default, Clone, Copy)]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
pub struct ContainerElement {
    pub orientation: Orientation,
    pub spacing: f32,
    pub children: Vec<Node>,
}

impl ContainerElement {
    #[must_use]
    pub const fn column() -> Self {
        Self {
            orientation: Orientation::Vertical,
            spacing: 0.0,
            children: Vec::new(),
        }
    }

    #[must_use]
    pub const fn row() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            spacing: 0.0,
            children: Vec::new(),
        }
    }
}

impl Layout<Context> for ContainerElement {
    fn measure(&self, context: &Context, style: &Style, parent: &Rect) -> MeasureNode {
        let mut node = MeasureNode::new(style, parent, parent.size);

        let mut size = Size::default();

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
