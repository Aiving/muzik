use skia_safe::font_style::{Slant, Weight};

use crate::muzui::language::{
    lexer::Token,
    parser::{Parse, Parser, Result},
};

#[derive(Debug, Default, Clone, Copy)]
pub enum FontWeight {
    Light,
    #[default]
    Normal,
    Bold,
}

impl From<FontWeight> for Weight {
    fn from(value: FontWeight) -> Self {
        match value {
            FontWeight::Light => Self::LIGHT,
            FontWeight::Normal => Self::NORMAL,
            FontWeight::Bold => Self::BOLD,
        }
    }
}

impl Parse for FontWeight {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume_map(|token| {
            token.try_as_ident().and_then(|ident| match ident {
                "light" => Some(Self::Light),
                "normal" => Some(Self::Normal),
                "bold" => Some(Self::Bold),
                _ => None,
            })
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum FontSlant {
    #[default]
    Upright,
    Italic,
    Oblique,
}

impl From<FontSlant> for Slant {
    fn from(value: FontSlant) -> Self {
        match value {
            FontSlant::Upright => Self::Upright,
            FontSlant::Italic => Self::Italic,
            FontSlant::Oblique => Self::Oblique,
        }
    }
}

impl Parse for FontSlant {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume_map(|token| {
            token.try_as_ident().and_then(|ident| match ident {
                "upright" => Some(Self::Upright),
                "italic" => Some(Self::Italic),
                "oblique" => Some(Self::Oblique),
                _ => None,
            })
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FontSize {
    pub size: f32,
}

impl FontSize {
    #[must_use]
    pub const fn new(size: f32) -> Self {
        Self { size }
    }
}

impl Default for FontSize {
    fn default() -> Self {
        Self::new(12.0)
    }
}

impl Parse for FontSize {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume_map(Token::try_as_f32).map(Self::new)
    }
}

#[derive(Debug, Clone)]
pub struct FontFamily {
    pub family: String,
}

impl FontFamily {
    pub fn new(family: impl Into<String>) -> Self {
        Self {
            family: family.into(),
        }
    }
}

impl Default for FontFamily {
    fn default() -> Self {
        Self::new("Arial")
    }
}

impl Parse for FontFamily {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser
            .consume_if(Token::is_string)
            .map(Token::into_string)
            .map(Self::new)
    }
}
