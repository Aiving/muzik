use crate::{
    graphics::Context,
    layout::{Layout, MeasureNode, Measurer, Rect},
    styling::Style,
    Node,
};

#[derive(Debug, Clone)]
pub struct Masonry {
    pub spacing: f32,
    pub item_width: f32,
    pub children: Vec<Node>,
}

impl Masonry {
    #[must_use]
    pub const fn new(item_width: f32) -> Self {
        Self {
            spacing: 0.0,
            item_width,
            children: Vec::new(),
        }
    }

    fn columns(&self, available_width: f32) -> f32 {
        (available_width / (self.item_width + self.spacing)).floor()
    }

    fn item_width(&self, available_width: f32) -> f32 {
        let columns = self.columns(available_width);

        self.spacing.mul_add(-(columns - 1.0), available_width) / columns
    }
}

impl Layout<Context> for Masonry {
    fn measure(&self, context: &Context, style: &Style, parent: &Rect) -> MeasureNode {
        let mut node = MeasureNode::from_parent(style, parent);

        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let columns = self.columns(node.inner.size.width) as usize;
        let item_width = self.item_width(node.inner.size.width);

        for child in self
            .children
            .chunks(columns)
            .fold(Vec::new(), |mut rows, row| {
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
                            .and_then(|last: &Vec<MeasureNode>| last.get(x))
                            .map(|last| last.outer.origin.y + last.outer.size.height + self.spacing)
                            .unwrap_or_default();
                        #[allow(clippy::cast_precision_loss)]
                        let x = (item_width + self.spacing).mul_add(x as f32, node.inner.origin.x);

                        let style = child.get_style();
                        let mut base = MeasureNode::from_parent(
                            style,
                            &Rect::from_xywh(x, y, item_width, 0.0),
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
