use std::collections::HashMap;

use muzui_layout::{Layout, MeasureNode, Measurer, Rect};
use muzui_styling::{Length, Style};

use crate::{graphics::Context, Node};

#[derive(Debug, Copy, Clone)]
pub enum GridLength {
    Auto,
    Px(f32),
    Weight(f32),
}

impl From<Length> for GridLength {
    fn from(value: Length) -> Self {
        match value {
            Length::Px(value) => Self::Px(value),
            _ => Self::Auto,
        }
    }
}

impl Default for GridLength {
    fn default() -> Self {
        Self::Weight(1.0)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Row {
    pub height: GridLength,
}

// #[must_use]
// pub const fn row_def(height: GridLength) -> Row {
//     Row { height }
// }

#[derive(Debug, Default, Copy, Clone)]
pub struct Column {
    pub width: GridLength,
}

// #[must_use]
// pub const fn column_def(width: GridLength) -> Column {
//     Column { width }
// }

#[derive(Debug, Default, Clone)]
pub struct GridElement {
    pub rows: Vec<Row>,
    pub columns: Vec<Column>,
    pub spacing: f32,
    pub children: Vec<Node>,
}

impl Layout<Context> for GridElement {
    #[allow(clippy::too_many_lines)]
    fn measure(&self, context: &Context, style: &Style, parent: &Rect) -> MeasureNode {
        assert!(
            style.width.as_ref().is_some_and(|width| !width.is_auto()),
            "the grid must have an explicitly specified width"
        );

        assert!(
            style.height.as_ref().is_some_and(|width| !width.is_auto()),
            "the grid must have an explicitly specified height"
        );

        let mut root = MeasureNode::new(style, parent, parent.size);

        let mut rows = HashMap::new();
        let mut columns = HashMap::new();

        let mut available_width =
            ((self.columns.len().max(1) - 1) as f32).mul_add(-self.spacing, root.inner.width());
        let mut available_height =
            ((self.rows.len().max(1) - 1) as f32).mul_add(-self.spacing, root.inner.height());

        let mut rows_weight = 0.0;
        let mut columns_weight = 0.0;

        for (i, row) in self.rows.iter().enumerate() {
            match row.height {
                GridLength::Auto => {
                    let size = self
                        .children
                        .iter()
                        .filter(|node| {
                            let style = node.get_style();

                            style.row == i && style.row_span == 1
                        })
                        .fold(0.0f32, |init, node| {
                            init.max(node.measure(context, &root.inner).outer.size.height)
                        });

                    rows.insert(i, size);

                    available_height -= size;
                }
                GridLength::Px(size) => available_height -= size,
                GridLength::Weight(weight) => rows_weight += weight,
            }
        }

        for (i, column) in self.columns.iter().enumerate() {
            match column.width {
                GridLength::Auto => {
                    let size = self
                        .children
                        .iter()
                        .filter(|node| {
                            let style = node.get_style();

                            style.column == i && style.column_span == 1
                        })
                        .fold(0.0f32, |init, node| {
                            init.max(node.measure(context, &root.inner).outer.size.width)
                        });

                    columns.insert(i, size);

                    available_height -= size;
                }
                GridLength::Px(size) => available_width -= size,
                GridLength::Weight(weight) => columns_weight += weight,
            }
        }

        let base_height = available_height / rows_weight;
        let base_width = available_width / columns_weight;

        let rows = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                (
                    i,
                    match row.height {
                        GridLength::Auto => rows[&i],
                        GridLength::Px(size) => size,
                        GridLength::Weight(weight) => base_height * weight,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let columns = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, column)| {
                (
                    i,
                    match column.width {
                        GridLength::Auto => columns[&i],
                        GridLength::Px(size) => size,
                        GridLength::Weight(weight) => base_width * weight,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        for child in &self.children {
            let Style {
                row,
                row_span,
                column,
                column_span,
                ..
            } = child.get_style();

            let x = (0..*column).fold(
                (*column as f32).mul_add(self.spacing, root.inner.origin.x),
                |init, column| init + columns[&column],
            );
            let y = (0..*row).fold(
                (*row as f32).mul_add(self.spacing, root.inner.origin.y),
                |init, row| init + rows[&row],
            );
            let width =
                (*column..*column + *column_span).fold(0.0, |init, column| init + columns[&column]);
            let height = (*row..*row + *row_span).fold(0.0, |init, row| init + rows[&row]);

            println!("{width}x{height} at {x}x{y} [{row}->{row_span} x {column}->{column_span}]");

            let child = child.measure(context, &Rect::from_xywh(x, y, width, height));

            root.children.push(child);
        }

        root
    }
}
