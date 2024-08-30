use super::lexer::Token;
use std::{error::Error, fmt, iter::Peekable, ops::Range, vec::IntoIter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for ParseError {}

impl ParseError {
    pub fn new<T: Into<String>>(value: T) -> Self {
        Self(value.into())
    }

    fn unexpected_token(token: Option<&Token>) -> Self {
        token.map_or_else(
            || Self("Unexpected nothing".to_string()),
            |token| Self(format!("Unexpected {token}")),
        )
    }

    fn expected_token(expected: &Token, found: Option<&Token>) -> Self {
        found.map_or_else(
            || Self(format!("Expected {expected}, found nothing")),
            |found| Self(format!("Expected {expected}, found {found}")),
        )
    }

    fn expected_tokens(expected: &[Token], found: Option<&Token>) -> Self {
        let expected = match expected.len() {
            0 => "nothing".into(),
            1 => expected[0].to_string(),
            value => {
                format!(
                    "{}{} or {}",
                    expected[0],
                    expected[1..value - 1]
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    expected[value - 1]
                )
            }
        };

        found.map_or_else(
            || Self(format!("Expected {expected}, found nothing")),
            |found| Self(format!("Expected {expected}, found {found}")),
        )
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    #[must_use]
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    /// Consumes the current token only if it exists and is equal to `value`.
    pub fn try_consume(&mut self, value: &Token) -> bool {
        if self.peek().is_some_and(|v| v == value) {
            self.next();

            true
        } else {
            false
        }
    }

    /// Checks if the next token exists and it is equal to `value`.
    pub fn check(&mut self, value: &Token) -> bool {
        self.peek().is_some_and(|v| v == value)
    }

    /// Returns the `bool` result of `func` if the next token exists.
    pub fn check_if<F: Fn(&Token) -> bool>(&mut self, func: F) -> bool {
        self.peek().is_some_and(func)
    }

    /// Consumes the current token if it exists and is equal to `value`, otherwise returning `ParseError`.
    ///
    /// # Errors
    ///
    /// Returns error if current token is not equal to `value`
    pub fn consume(&mut self, value: &Token) -> Result<Token> {
        self.next_if(|current| current == value)
            .map_or_else(|| Err(ParseError::expected_token(value, self.peek())), Ok)
    }

    /// Consumes the current token if it exists and is equal to one of the values inside `values`, otherwise returning `ParseError`.
    ///
    /// # Errors
    ///
    /// Returns error if current token is not equal to one of the tokens inside `values`
    pub fn consume_one_of(&mut self, values: &[Token]) -> Result<Token> {
        self.next_if(|value| values.contains(value))
            .map_or_else(|| Err(ParseError::expected_tokens(values, self.peek())), Ok)
    }

    /// Consumes the current token if it exists and the result of `func` is `true`, otherwise returning `ParseError`.
    ///
    /// # Errors
    ///
    /// Returns error if result of the `func` is false
    pub fn consume_if<F: Fn(&Token) -> bool>(&mut self, func: F) -> Result<Token> {
        self.next_if(func)
            .map_or_else(|| Err(ParseError::unexpected_token(self.peek())), Ok)
    }

    /// Consumes the current token if it exists and the result of the `func` is `Some(T)`, otherwise returning `ParseError`.
    ///
    /// # Errors
    ///
    /// Returns error if there is no token or result of the `func` is None
    pub fn consume_map<T, F: Fn(&Token) -> Option<T>>(&mut self, func: F) -> Result<T> {
        if let Some(value) = self.peek().and_then(func) {
            self.next();

            Ok(value)
        } else {
            Err(ParseError::unexpected_token(self.peek()))
        }
    }

    /// Consumes the current token and returns it wrapped in `Some` if it exists, otherwise returning `None`.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    /// Peeks the current token and returns a reference to it wrapped in `Some` if it exists, otherwise returning `None`.
    pub fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    /// Consumes the current token and returns it wrapped in `Some` if the result of the `func` function is `true`, otherwise returning `None`.
    pub fn next_if<F: Fn(&Token) -> bool>(&mut self, func: F) -> Option<Token> {
        if self.check_if(func) {
            self.next()
        } else {
            None
        }
    }
}

pub trait Parse: Sized {
    /// # Errors
    ///
    /// Returns error if parsing failed
    fn parse(parser: &mut Parser) -> Result<Self>;
}

impl Parse for f32 {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume_if(Token::is_f32).map(Token::into_f32)
    }
}

impl Parse for i64 {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume_if(Token::is_i64).map(Token::into_i64)
    }
}

impl Parse for String {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume_if(Token::is_string).map(Token::into_string)
    }
}

impl Parse for bool {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.consume_if(Token::is_bool).map(Token::into_bool)
    }
}

impl Parse for Range<usize> {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let start = parser
            .consume_map(|token| token.try_as_i64().and_then(|value| value.try_into().ok()))?;

        parser.consume(&Token::Dot)?;
        parser.consume(&Token::Dot)?;

        parser
            .consume_map(|token| token.try_as_i64().and_then(|value| value.try_into().ok()))
            .map(|end| start..end)
    }
}
