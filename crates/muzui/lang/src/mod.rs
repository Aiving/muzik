pub mod lexer;
pub mod parser;
pub mod program;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use material_colors::{color::Argb, theme::ThemeBuilder};

    use crate::muzui::{
        language::{parser::Parser, program::parse_node},
        RenderContext,
    };

    use super::lexer::Lexer;

    const DATA: &str = "Column {
        padding: 8,
        background: theme.surface_container,
        spacing: 8,
        width: 50%,

        Row {
            height: 200,
            width: 100%,
            background: theme.primary_container,
            corner-radius: 50,
            
            Row {
                height: 50%,
                width: 50%,
                background: theme.tertiary_container,
                corner-radius: 50,
            }
        }

        Row {
            height: 200,
            width: 200,
            background: theme.primary_container,
            corner-radius: 50
        }

        Text(\"АЛО {sub.wtf} ВАШИХ\") {
            background: theme.primary_container,
            color: theme.on_primary_container,
            font-family: \"Source Code Pro\",
            font-size: 32,
            font-weight: bold
        }

        Text(\"АЛО {xd} ВАШИХ\") {
            background: theme.primary_container,
            color: theme.on_primary_container,
            font-family: \"Source Code Pro\",
            font-size: 32,
            font-weight: bold
        }

        Image(\"/home/aiving/Pictures/e4c2e8651098c6989b2e47b1b910e788.png\") {
            height: 200,
            width: 200,
            position: absolute,
            x: 100,
            y: 50,
            corner-radius: 50
        }
    }";

    #[test]
    fn test_lexer() {
        println!("{:#?}", Lexer::parse(DATA));
    }

    #[test]
    fn test_parser() {
        let mut parser = Parser::new(Lexer::parse(DATA));
        let theme = ThemeBuilder::with_source(Argb::from_u32(0xFFFF0000))
            .build()
            .schemes
            .dark
            .into_iter()
            .collect::<HashMap<_, _>>();

        println!("{:#?}", parse_node(&mut parser, &theme, None));
    }

    #[test]
    fn test_result() {
        let mut parser = Parser::new(Lexer::parse(DATA));
        let theme = ThemeBuilder::with_source(Argb::from_u32(0xFFFF0000))
            .build()
            .schemes
            .dark
            .into_iter()
            .collect::<HashMap<_, _>>();
        let node = parse_node(&mut parser, &theme, None).expect("failed to parse node");

        let mut context = RenderContext::new(1000, 1000).unwrap();

        context.render(&node);

        if let Some(data) = context.encode() {
            std::fs::write("/home/aiving/test.png", &data).unwrap();
        }
    }
}
