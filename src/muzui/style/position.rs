use crate::muzui::language::parser::{Parse, Parser, Result};

#[derive(Debug, Default, Clone, Copy)]
pub enum Position {
    Absolute,
    #[default]
    Relative,
}

impl Position {
    /// Returns `true` if the position is [`Absolute`].
    ///
    /// [`Absolute`]: Position::Absolute
    #[must_use]
    pub const fn is_absolute(&self) -> bool {
        matches!(self, Self::Absolute)
    }

    /// Returns `true` if the position is [`Relative`].
    ///
    /// [`Relative`]: Position::Relative
    #[must_use]
    pub const fn is_relative(&self) -> bool {
        matches!(self, Self::Relative)
    }
}

impl Parse for Position {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume_map(|token| {
            token.try_as_ident().and_then(|ident| match ident {
                "absolute" => Some(Self::Absolute),
                "relative" => Some(Self::Relative),
                _ => None,
            })
        })
    }
}
