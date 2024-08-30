#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

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
    pub use crate::{layout::*, macros::*, styling::*, Node, RenderContext};
}

#[cfg(test)]
mod tests {
    use crate as muzui;
    use muzui::prelude::*;

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
}
