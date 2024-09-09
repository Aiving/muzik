#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions, clippy::cast_precision_loss)]

use self::styling::Length;
pub use self::{elements::*, graphics::RenderContext, node::*};

mod elements;
mod graphics;
mod node;

pub mod geometry {
    pub use muzui_geometry::*;
}

pub mod layout {
    pub use muzui_layout::*;
}

pub mod macros {
    pub use muzui_macros::*;
}

pub mod styling {
    pub use muzui_styling::*;
}

pub mod prelude {
    pub use crate::{
        layout::*, macros::*, styling::*, FloatExt, FloatGridLengthExt, FloatLengthExt, Node,
        RenderContext,
    };
}

pub trait FloatExt<T> {
    fn px(self) -> T;
}

pub trait FloatLengthExt {
    fn percent(self) -> Length;
}

pub trait FloatGridLengthExt {
    fn weight(self) -> GridLength;
}

impl FloatExt<Length> for i32 {
    fn px(self) -> Length {
        Length::Px(self as f32)
    }
}

impl FloatExt<GridLength> for i32 {
    fn px(self) -> GridLength {
        GridLength::Px(self as f32)
    }
}

impl FloatLengthExt for i32 {
    fn percent(self) -> Length {
        Length::Percent(self as f32)
    }
}

impl FloatGridLengthExt for i32 {
    fn weight(self) -> GridLength {
        GridLength::Weight(self as f32)
    }
}

impl FloatExt<Length> for f32 {
    fn px(self) -> Length {
        Length::Px(self)
    }
}

impl FloatExt<GridLength> for f32 {
    fn px(self) -> GridLength {
        GridLength::Px(self)
    }
}

impl FloatLengthExt for f32 {
    fn percent(self) -> Length {
        Length::Percent(self)
    }
}

impl FloatGridLengthExt for f32 {
    fn weight(self) -> GridLength {
        GridLength::Weight(self)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate as muzui;
    use material_colors::{color::Argb, scheme::Scheme, theme::ThemeBuilder};
    use muzui::prelude::*;

    trait ArgbExt {
        fn as_color(&self) -> Color;
    }

    impl ArgbExt for Argb {
        fn as_color(&self) -> Color {
            Color::from_rgba(self.red, self.green, self.blue, self.alpha)
        }
    }

    #[test]
    fn test_macros() {
        #[allow(non_snake_case)]
        fn Button(_: Style) -> Node {
            ui! {
                Column {
                    height: px(100.0),
                    width: px(100.0),
                    background: 0x00FF_FF00,

                    Row {
                        spacing: 8.0,

                        Text("Hello")
                        Text("World") {
                            font_family: "JetBrainsMono Nerd Font Mono",
                        }
                    }
                }
            }
        }

        let node = ui! {
            Button {
                height: px(100.0),
                width: px(100.0),
                background: 0x00FF_FF00,
            }
        };

        println!("{node:#?}");
    }

    #[test]
    fn test_graphics() {
        #[allow(non_snake_case, clippy::needless_pass_by_value)]
        fn Cell(style: Style, theme: &Scheme) -> Node {
            ui! {
                Column {
                    height: 100.percent(),
                    width: 100.percent(),
                    background: theme.primary_container.as_u32(),
                    row: style.row,
                    column: style.column,
                    padding: 8.0,
                    corner_radius: 24.0,

                    Text("Hello World") {
                        font_family: "JetBrainsMono Nerd Font Mono",
                        color: theme.on_primary_container.as_u32(),
                    }
                }
            }
        }

        let theme = ThemeBuilder::with_source(Argb::new(255, 50, 100, 255))
            .build()
            .schemes
            .dark;

        let node = ui! {
            Grid {
                padding: 8.0,
                height: 100.percent(),
                width: 100.percent(),
                background: theme.surface_container_high.as_u32(),
                columns: [
                    1.weight(),
                    1.weight(),
                    3.weight(),
                ],
                rows: [
                    1.weight(),
                    1.weight(),
                ],
                spacing: 8.0,

                Cell(&theme) { row: 0, column: 0 }
                Cell(&theme) { row: 0, column: 1 }
                Cell(&theme) { row: 0, column: 2 }
                Cell(&theme) { row: 1, column: 2 }
            }
        };

        let mut context = RenderContext::new(600, 100).unwrap();

        context.render(&node);

        if let Some(data) = context.encode() {
            fs::write("./image.png", data).unwrap();
        }
    }
}
