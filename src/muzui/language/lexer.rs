use std::{
    fmt::{self, Write},
    iter, mem,
};

#[derive(Debug, PartialEq, Clone)]
pub enum StringPart {
    String(String),
    Formatted(Vec<Token>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    FormattedString(Vec<StringPart>),
    String(String),
    Float(f32),
    Integer(i64),
    Boolean(bool),
    BracketOpen,
    BracketClose,
    BraceOpen,
    BraceClose,
    ParenOpen,
    ParenClose,
    Colon,
    Minus,
    Plus,
    Slash,
    Star,
    Pound,
    Percent,
    Comma,
    Dot,
    Unknown(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(value) | Self::String(value) => value.fmt(f),
            Self::Float(value) => value.fmt(f),
            Self::Integer(value) => value.fmt(f),
            Self::Boolean(value) => value.fmt(f),
            Self::BracketOpen => f.write_char('['),
            Self::BracketClose => f.write_char(']'),
            Self::BraceOpen => f.write_char('{'),
            Self::BraceClose => f.write_char('}'),
            Self::ParenOpen => f.write_char('('),
            Self::ParenClose => f.write_char(')'),
            Self::Colon => f.write_char(':'),
            Self::Minus => f.write_char('-'),
            Self::Plus => f.write_char('+'),
            Self::Slash => f.write_char('/'),
            Self::Star => f.write_char('*'),
            Self::Pound => f.write_char('#'),
            Self::Percent => f.write_char('%'),
            Self::Dot => f.write_char('.'),
            Self::Comma => f.write_char(','),
            Self::Unknown(value) => write!(f, "unknown token {value}"),
            Self::FormattedString(parts) => parts.iter().try_fold((), |prev, value| match value {
                StringPart::String(value) => value.fmt(f),
                StringPart::Formatted(formatted) => {
                    formatted.iter().try_fold(prev, |(), token| token.fmt(f))
                }
            }),
        }
    }
}

impl Token {
    pub fn ident<T: Into<String>>(value: T) -> Self {
        Self::Ident(value.into())
    }

    #[must_use]
    pub const fn is_ident(&self) -> bool {
        matches!(self, Self::Ident(_))
    }

    #[must_use]
    pub const fn is_f32(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    #[must_use]
    pub const fn is_i64(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    #[must_use]
    pub const fn is_i64_or_f32(&self) -> bool {
        matches!(self, Self::Integer(_) | Self::Float(_))
    }

    #[must_use]
    pub fn into_string(self) -> String {
        if let Self::String(value) = self {
            value
        } else {
            unreachable!()
        }
    }

    #[must_use]
    pub fn into_ident(self) -> String {
        if let Self::Ident(value) = self {
            value
        } else {
            unreachable!()
        }
    }

    #[must_use]
    pub fn into_f32(self) -> f32 {
        if let Self::Float(value) = self {
            value
        } else if let Self::Integer(value) = self {
            value as f32
        } else {
            unreachable!()
        }
    }

    #[must_use]
    pub fn into_i64(self) -> i64 {
        if let Self::Integer(value) = self {
            value
        } else {
            unreachable!()
        }
    }

    #[must_use]
    pub fn into_bool(self) -> bool {
        if let Self::Boolean(value) = self {
            value
        } else {
            unreachable!()
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        if let Self::String(value) = self {
            value.as_str()
        } else {
            unreachable!()
        }
    }

    #[must_use]
    pub const fn try_as_bool(&self) -> Option<bool> {
        if let Self::Boolean(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[must_use]
    pub fn try_as_str(&self) -> Option<&str> {
        if let Self::String(value) = self {
            Some(value.as_str())
        } else {
            None
        }
    }

    #[must_use]
    pub fn try_as_ident(&self) -> Option<&str> {
        if let Self::Ident(value) = self {
            Some(value.as_str())
        } else {
            None
        }
    }

    #[must_use]
    pub const fn try_as_f32(&self) -> Option<f32> {
        if let Self::Float(value) = self {
            Some(*value)
        } else if let Self::Integer(value) = self {
            Some(*value as f32)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn try_as_i64(&self) -> Option<i64> {
        if let Self::Integer(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    #[must_use]
    pub fn try_as_u8(&self) -> Option<u8> {
        if let Self::Integer(value) = self {
            u8::try_from(*value).ok()
        } else {
            None
        }
    }

    /// Returns `true` if the token is [`String`].
    ///
    /// [`String`]: Token::String
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns `true` if the token is [`Boolean`].
    ///
    /// [`Boolean`]: Token::Boolean
    #[must_use]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    /// Returns `true` if the token is [`FormattedString`].
    ///
    /// [`FormattedString`]: Token::FormattedString
    #[must_use]
    pub const fn is_formatted_string(&self) -> bool {
        matches!(self, Self::FormattedString(_))
    }

    #[must_use]
    pub fn into_formatted_string(self) -> Vec<StringPart> {
        if let Self::FormattedString(v) = self {
            v
        } else {
            unreachable!()
        }
    }
}

pub struct Lexer;

impl Lexer {
    /// # Panics
    ///
    /// Can panic if number failed to parse
    pub fn parse<T: AsRef<str>>(data: T) -> Vec<Token> {
        let mut tokens = vec![];
        let mut chars = data.as_ref().chars().peekable();

        while let Some(character) = chars.next() {
            match character {
                character if character.is_ascii_whitespace() => continue,
                '[' => tokens.push(Token::BracketOpen),
                ']' => tokens.push(Token::BracketClose),
                '{' => tokens.push(Token::BraceOpen),
                '}' => tokens.push(Token::BraceClose),
                '(' => tokens.push(Token::ParenOpen),
                ')' => tokens.push(Token::ParenClose),
                'A'..='z' => {
                    let ident = iter::once(character)
                        .chain(iter::from_fn(|| {
                            chars
                                .by_ref()
                                .next_if(|s| s.is_ascii_alphanumeric() || s == &'-' || s == &'_')
                        }))
                        .collect::<String>();

                    match ident.as_str() {
                        "true" => tokens.push(Token::Boolean(true)),
                        "false" => tokens.push(Token::Boolean(false)),
                        _ => tokens.push(Token::Ident(ident)),
                    }
                }
                '0'..='9' => {
                    let mut value = iter::once(character)
                        .chain(iter::from_fn(|| {
                            chars.by_ref().next_if(char::is_ascii_digit)
                        }))
                        .collect::<String>();

                    if chars.peek().is_some_and(|&character| character == '.') {
                        chars.next();

                        let after = iter::from_fn(|| chars.by_ref().next_if(char::is_ascii_digit))
                            .collect::<String>();

                        if after.is_empty() {
                            tokens.push(Token::Integer(value.parse().unwrap()));
                            tokens.push(Token::Dot);
                        } else {
                            value.push('.');
                            value.push_str(&after);

                            tokens.push(Token::Float(value.parse().unwrap()));
                        }

                        continue;
                    }

                    tokens.push(Token::Integer(value.parse().unwrap()));
                }
                ':' => tokens.push(Token::Colon),
                '+' => tokens.push(Token::Plus),
                '-' => {
                    if chars.peek().is_some_and(char::is_ascii_digit) {
                        let value = iter::once(character)
                            .chain(iter::from_fn(|| {
                                chars.by_ref().next_if(|s| s.is_ascii_digit() || s == &'.')
                            }))
                            .collect::<String>();

                        if value.contains('.') {
                            tokens.push(Token::Float(value.parse().unwrap()));
                        } else {
                            tokens.push(Token::Integer(value.parse().unwrap()));
                        }
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => tokens.push(Token::Star),
                '/' => tokens.push(Token::Slash),
                '#' => {
                    tokens.push(Token::Pound);

                    if chars.peek().is_some_and(char::is_ascii_alphanumeric) {
                        tokens.push(Token::Ident(
                            iter::from_fn(|| chars.by_ref().next_if(char::is_ascii_alphanumeric))
                                .collect::<String>(),
                        ));
                    }
                }
                '%' => tokens.push(Token::Percent),
                '.' => tokens.push(Token::Dot),
                ',' => tokens.push(Token::Comma),
                '"' => {
                    let mut data = String::new();
                    let mut datas = Vec::new();
                    let mut formatted = Vec::new();

                    while let Some(character) = chars.next_if(|&s| s != '"') {
                        if character == '\\' {
                            chars.next_if(|&s| s == '"');

                            data.push('"');
                        }

                        if character == '{' {
                            datas.push(mem::take(&mut data));

                            formatted.push(
                                iter::from_fn(|| chars.next_if(|&s| s != '{' && s != '}'))
                                    .collect::<String>(),
                            );

                            chars.next_if(|&s| s == '}');
                        } else {
                            data.push(character);
                        }
                    }

                    chars.next_if(|&s| s == '"');

                    if datas.is_empty() && formatted.is_empty() {
                        tokens.push(Token::String(data));
                    } else {
                        let mut parts = datas
                            .into_iter()
                            .map(StringPart::String)
                            .zip(
                                formatted
                                    .into_iter()
                                    .map(|value| StringPart::Formatted(Self::parse(value))),
                            )
                            .fold(Vec::new(), |prev, (a, b)| [prev, vec![a, b]].concat());

                        parts.push(StringPart::String(data));

                        tokens.push(Token::FormattedString(parts));
                    }
                }
                character => tokens.push(Token::Unknown(character)),
            }
        }

        tokens
    }
}
