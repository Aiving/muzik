use std::{collections::HashMap, ops::Range};

use material_colors::color::Argb;

use crate::muzui::{
    layout::{Length, Orientation},
    style::{
        font::{FontSlant, FontWeight},
        position::Position,
        thickness::Thickness,
    },
};

use super::{
    lexer::{StringPart, Token},
    parser::{Parse, ParseError, Parser, Result},
};

pub enum Value {
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    String(String),
    Boolean(bool),
    BinaryData(Vec<u8>),
}

impl Value {
    fn into_string(self) -> String {
        match self {
            Self::I8(value) => value.to_string(),
            Self::U8(value) => value.to_string(),
            Self::I16(value) => value.to_string(),
            Self::U16(value) => value.to_string(),
            Self::I32(value) => value.to_string(),
            Self::U32(value) => value.to_string(),
            Self::I64(value) => value.to_string(),
            Self::U64(value) => value.to_string(),
            Self::String(value) => value,
            Self::Boolean(value) => value.to_string(),
            Self::BinaryData(_) => "<binary>".to_string(),
        }
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Self::I8(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::U8(value)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Self::I16(value)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Self::U16(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::I32(value)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<&String> for Value {
    fn from(value: &String) -> Self {
        Self::String(value.clone())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

pub enum Index {
    String(String),
    Number(usize),
}

pub trait Indexable {
    fn index(&self, keys: Vec<Index>) -> Option<Value>;
}

enum ElementName {
    Row,
    Column,
    Masonry,
    Container,
    Text,
    Image,
}

fn parse_index(parser: &mut Parser) -> Result<Vec<Index>> {
    let mut indexes = vec![Index::String(
        parser.consume_if(Token::is_ident).map(Token::into_ident)?,
    )];

    while let Ok(indexer) = parser.consume_one_of(&[Token::Dot, Token::BracketOpen]) {
        indexes.push(if matches!(indexer, Token::BracketOpen) {
            let value =
                Index::Number(parser.consume_map(|token| {
                    token.try_as_i64().and_then(|value| value.try_into().ok())
                })?);

            parser.consume(&Token::BracketClose)?;

            value
        } else {
            Index::String(parser.consume_if(Token::is_ident).map(Token::into_ident)?)
        });
    }

    Ok(indexes)
}

#[derive(Debug)]
struct Attribute {
    name: String,
    value: Expression,
}

#[derive(Debug)]
struct IndexExpression {
    target: Expression,
    index: Expression,
}

impl Parse for IndexExpression {
    fn parse(parser: &mut Parser) -> Result<Self> {
        println!("parsing index");

        let target = Expression::parse(parser)?;

        Expression::parse(parser).map(|index| Self { target, index })
    }
}

#[derive(Debug)]
enum Number {
    Float(f32),
    Int(i64),
}

impl Parse for Number {
    fn parse(parser: &mut Parser) -> Result<Self> {
        println!("parsing number");

        f32::parse(parser)
            .map(Self::Float)
            .or_else(|_| i64::parse(parser).map(Self::Int))
    }
}

#[derive(Debug)]
enum Literal {
    Number(Number),
    Length(Length),
    String(String),
    Boolean(bool),
    Range(Range<usize>),
}

impl Parse for Literal {
    fn parse(parser: &mut Parser) -> Result<Self> {
        println!("parsing literal");

        bool::parse(parser)
            .map(Self::Boolean)
            .or_else(|_| Range::parse(parser).map(Self::Range))
            .or_else(|_| Number::parse(parser).map(Self::Number))
            .or_else(|_| Length::parse(parser).map(Self::Length))
            .or_else(|_| String::parse(parser).map(Self::String))
    }
}

#[derive(Debug)]
enum Formatting {
    String(String),
    Expression(Expression),
}

#[derive(Debug)]
enum Expression {
    Position(Position),
    Orientation(Orientation),
    FontSlant(FontSlant),
    FontWeight(FontWeight),
    Thickness(Thickness),
    Index(Box<IndexExpression>),
    Ident(String),
    Literal(Literal),
    FormattedString(Vec<Formatting>),
}

impl Parse for Attribute {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let name = parser.consume_if(Token::is_ident).map(Token::into_ident)?;

        parser.consume(&Token::Colon)?;

        Expression::parse(parser).map(|value| Self { name, value })
    }
}

impl Parse for Expression {
    fn parse(parser: &mut Parser) -> Result<Self> {
        println!("parsing expression");

        FontWeight::parse(parser)
            .map(Self::FontWeight)
            .or_else(|_| FontSlant::parse(parser).map(Self::FontSlant))
            .or_else(|_| Position::parse(parser).map(Self::Position))
            .or_else(|_| Orientation::parse(parser).map(Self::Orientation))
            .or_else(|_| {
                IndexExpression::parse(parser)
                    .map(Box::new)
                    .map(Self::Index)
            })
            .or_else(|_| Literal::parse(parser).map(Self::Literal))
            .or_else(|_| Thickness::parse(parser).map(Self::Thickness))
            .or_else(|_| {
                let parts = parser
                    .consume_if(Token::is_formatted_string)
                    .map(Token::into_formatted_string)?;

                let mut data = Vec::new();

                for part in parts {
                    match part {
                        StringPart::String(value) => data.push(Formatting::String(value)),
                        StringPart::Formatted(value) => {
                            let mut parser = Parser::new(value);

                            data.push(Formatting::Expression(Self::parse(&mut parser)?));
                        }
                    }
                }

                Ok(Self::FormattedString(data))
            })
            .or_else(|_: ParseError| {
                parser
                    .consume_if(Token::is_ident)
                    .map(Token::into_ident)
                    .map(Self::Ident)
            })
    }
}

#[derive(Debug)]
struct ForStatement {
    name: String,
    target: Expression,
    body: Vec<Statement>,
}

#[derive(Debug)]
enum Statement {
    ForStatement(ForStatement),
    Node(Node),
}

impl Parse for ForStatement {
    fn parse(parser: &mut Parser) -> Result<Self> {
        println!("parsing for in");

        parser.consume(&Token::ident("for"))?;

        let name = parser.consume_if(Token::is_ident).map(Token::into_ident)?;

        parser.consume(&Token::ident("in"))?;

        let target = Expression::parse(parser)?;

        parser.consume(&Token::BraceOpen)?;

        let mut body = Vec::new();

        while !parser.check(&Token::BraceClose) {
            body.push(Statement::parse(parser)?);
        }

        parser.consume(&Token::BraceClose)?;

        Ok(Self { name, target, body })
    }
}

impl Parse for Statement {
    fn parse(parser: &mut Parser) -> Result<Self> {
        ForStatement::parse(parser)
            .map(Self::ForStatement)
            .or_else(|_| Node::parse(parser).map(Self::Node))
    }
}

#[derive(Debug)]
struct Node {
    name: String,
    args: Vec<Expression>,
    attributes: Vec<Attribute>,
    body: Vec<Statement>,
}

impl Parse for Node {
    fn parse(parser: &mut Parser) -> Result<Self> {
        println!("parsing node");

        let name = parser.consume_if(Token::is_ident).map(Token::into_ident)?;

        let mut args = Vec::new();

        if parser.try_consume(&Token::ParenOpen) {
            while !parser.check(&Token::ParenClose) {
                if !args.is_empty() {
                    parser.consume(&Token::Comma)?;
                }

                args.push(Expression::parse(parser)?);
            }

            parser.consume(&Token::ParenClose)?;
        }

        let mut attributes = Vec::new();
        let mut body = Vec::new();

        if parser.try_consume(&Token::BraceOpen) {
            while let Ok(attribute) = Attribute::parse(parser) {
                println!("parsing attribute");

                if !attributes.is_empty() {
                    parser.consume(&Token::Comma)?;
                }

                attributes.push(attribute);
            }

            while !parser.check(&Token::BraceClose) {
                println!("parsing statement");

                body.push(Statement::parse(parser)?);
            }

            parser.consume(&Token::BraceClose)?;
        }

        Ok(Self {
            name,
            args,
            attributes,
            body,
        })
    }
}

pub fn parse_node(
    _: &mut Parser,
    _: &HashMap<String, Argb>,
    _: Option<&dyn Indexable>,
) -> Result<crate::muzui::node::Node> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::muzui::language::{
        lexer::{Lexer, Token},
        parser::{Parse, Parser}, program::{Attribute, Expression, Statement},
    };

    use super::Node;

    #[test]
    fn test_node() {
        let mut parser = Parser::new(Lexer::parse(
            r#"Column {
  padding: 96 96 96 0,
  spacing: 48,
  background: theme.surface_container,
  width: 100%,
  height: 100%,

  Row {
    spacing: 48,

    Column {
      padding: 320 0 48 0,
      background: theme.surface_container_highest,
      corner-radius: 48,

      Text("{user_info.user_info.nickname}") {
        background: theme.primary_container,
        color: theme.primary,
        font-size: 96,
      }
    }

    Column {
      padding: 0 48,
      background: theme.surface_container_highest,
      corner-radius: 48,

      Text("{user_info.user_info.introduce} a") {
        color: theme.primary,
        font-size: 96,
      }
    }
  }

  Column {
    background: theme.surface_container_highest,
    width: 100%,
    height: 100%,
    corner-radius: 72 72 0 0,
    padding: 12 120 12 12,

    Text("{characters.list[0].weapon.name} a") {
      background: theme.primary_container,
      color: theme.primary,
      font-weight: bold,
      font-size: 80
    }

    Text("{characters.list[0].weapon.desc}") {
      color: theme.primary,
      font-size: 72
    }
  }

  Image("") {
    position: absolute,
    corner-radius: 72,
    width: 320,
    height: 320,
    x: 48,
    y: 48
  }
}"#,
        ));

        println!("parsing node");

        let name = parser.consume_if(Token::is_ident).map(Token::into_ident).unwrap();

        let mut args = Vec::new();

        if parser.try_consume(&Token::ParenOpen) {
            while !parser.check(&Token::ParenClose) {
                if !args.is_empty() {
                    parser.consume(&Token::Comma).unwrap();
                }

                args.push(Expression::parse(&mut parser).unwrap());
            }

            parser.consume(&Token::ParenClose).unwrap();
        }

        let mut attributes = Vec::new();
        let mut body = Vec::new();

        if parser.try_consume(&Token::BraceOpen) {
            while let Ok(attribute) = Attribute::parse(&mut parser) {
                println!("parsing attribute");

                if !attributes.is_empty() {
                    parser.consume(&Token::Comma).unwrap();
                }

                attributes.push(attribute);
            }

            while !parser.check(&Token::BraceClose) {
                println!("parsing statement");

                body.push(Statement::parse(&mut parser).unwrap());
            }

            parser.consume(&Token::BraceClose).unwrap();
        }

        println!("{:#?}", Node {
            name,
            args,
            attributes,
            body,
        });
    }
}
