use std::mem::zeroed;

use material_colors::color::Argb;

use crate::muzui::language::{
    lexer::Token,
    parser::{Parse, ParseError, Parser, Result},
};

fn parse_color_function<const S: usize>(parser: &mut Parser) -> Result<[u8; S]> {
    parser.consume(&Token::ParenOpen)?;

    let mut colors: [u8; S] = unsafe { zeroed() };

    for (index, color) in colors.iter_mut().enumerate() {
        if index != 0 {
            parser.try_consume(&Token::Comma);
        }

        *color = parser.consume_map(Token::try_as_u8)?;
    }

    parser.consume(&Token::ParenClose)?;

    Ok(colors)
}

impl Parse for Argb {
    fn parse(parser: &mut Parser) -> Result<Self> {
        if parser.try_consume(&Token::Pound) {
            parser
                .consume_map(|value| {
                    value.try_as_ident().and_then(|value| {
                        if value.len() == 6 {
                            u32::from_str_radix(&format!("FF{value}"), 16).ok()
                        } else {
                            u32::from_str_radix(value, 16).ok()
                        }
                    })
                })
                .map(Self::from_u32)
        }
        /* else if parser.try_consume(&Token::ident("theme")) {
            parser.consume(&Token::Dot)?;

            parser.consume_map(|token| {
                token
                    .try_as_ident()
                    .and_then(|value| scheme.get(value).copied())
            })
        }  */
        else if parser.try_consume(&Token::ident("argb")) {
            let [a, r, g, b] = parse_color_function(parser)?;

            Ok(Self::new(a, r, g, b))
        } else if parser.try_consume(&Token::ident("rgba")) {
            let [r, g, b, a] = parse_color_function(parser)?;

            Ok(Self::new(a, r, g, b))
        } else if parser.try_consume(&Token::ident("rgb")) {
            let [r, g, b] = parse_color_function(parser)?;

            Ok(Self::new(255, r, g, b))
        } else {
            Err(ParseError::new("failed to parse color"))
        }
    }
}
