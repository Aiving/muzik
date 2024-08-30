#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Paren},
    Expr, Ident, Result, Token,
};

struct Attribute {
    name: Ident,
    #[allow(dead_code)]
    colon_token: Token![:],
    value: Expr,
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            colon_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

enum NodeName {
    Row,
    Column,
    Masonry,
    Image,
    Text,
    Custom(Ident),
}

struct NodeArgs {
    #[allow(dead_code)]
    paren_token: Paren,
    values: Punctuated<Expr, Token![,]>,
}

impl Parse for NodeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            paren_token: parenthesized!(content in input),
            values: Punctuated::parse_separated_nonempty(&content)?,
        })
    }
}

struct NodeBody {
    #[allow(dead_code)]
    brace_token: Brace,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
}

impl Parse for NodeBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        let brace_token = braced!(content in input);

        let mut attributes = Vec::new();

        while content.peek(Ident) && content.peek2(Token![:])
            || content.peek(Token![,]) && content.peek2(Ident) && content.peek3(Token![:])
        {
            if !attributes.is_empty() {
                content.parse::<Token![,]>()?;
            }

            attributes.push(Attribute::parse(&content).expect("failed to parse attribute"));
        }

        let mut children = Vec::new();

        if content.parse::<Token![,]>().is_ok() {
            while !content.is_empty() {
                children.push(Node::parse(&content)?);
            }
        }

        Ok(Self {
            brace_token,
            attributes,
            children,
        })
    }
}

struct Node {
    name: NodeName,
    args: Option<NodeArgs>,
    body: Option<NodeBody>,
}

impl Parse for Node {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse::<Ident>().map(|name| {
                if name == "Row" {
                    NodeName::Row
                } else if name == "Column" {
                    NodeName::Column
                } else if name == "Masonry" {
                    NodeName::Masonry
                } else if name == "Image" {
                    NodeName::Image
                } else if name == "Text" {
                    NodeName::Text
                } else {
                    NodeName::Custom(name)
                }
            })?,
            args: if input.peek(Paren) {
                Some(NodeArgs::parse(input)?)
            } else {
                None
            },
            body: if input.peek(Brace) {
                Some(NodeBody::parse(input)?)
            } else {
                None
            },
        })
    }
}

impl From<Node> for TokenStream2 {
    fn from(value: Node) -> Self {
        let mut is_builtin = true;
        let name = match value.name {
            NodeName::Row => quote! { muzui::Node::row },
            NodeName::Column => quote! { muzui::Node::column },
            NodeName::Masonry => quote! { muzui::Node::masonry },
            NodeName::Image => quote! { muzui::Node::image },
            NodeName::Text => quote! { muzui::Node::text },
            NodeName::Custom(name) => {
                is_builtin = false;

                name.into_token_stream()
            }
        };

        let [attributes, children]: [Vec<_>; 2] = value
            .body
            .map(|body| {
                [
                    body.attributes
                        .into_iter()
                        .map(
                            |Attribute {
                                 name,
                                 colon_token: _,
                                 value,
                             }| {
                                quote! {
                                    .#name(#value)
                                }
                            },
                        )
                        .collect(),
                    body.children.into_iter().map(Into::into).collect(),
                ]
            })
            .unwrap_or_default();

        let args = if let Some(args) = value.args {
            let args = args.values.into_token_stream();

            if is_builtin {
                quote! { (#args) }
            } else {
                quote! { (muzui::styling::StyleBuilder::new() #(#attributes)* .build(), #args) }
            }
        } else if is_builtin {
            quote! { () }
        } else {
            quote! { (muzui::styling::StyleBuilder::new() #(#attributes)* .build()) }
        };

        let builder = if is_builtin {
            quote! { #name #args }
        } else {
            quote! { muzui::Node::into_builder(#name #args) }
        };

        quote! {
            #builder #(#attributes)* .children(vec![#(#children),*]).build()
        }
    }
}

/// A macro that simplifies the creation of layouts for UI's.
///
/// # Examples
///
/// ```rust
/// use muzui_macros::ui;
///
/// let node = ui! {
///     Column {
///         height: px(100.0),
///         width: px(100.0),
///         background: 0xFFFF00,
///
///         Row {
///             spacing: 8.0,
///
///             Text("Hello")
///             Text("World")
///         }
///     }
/// };
/// ```
#[proc_macro]
pub fn ui(input: TokenStream) -> TokenStream {
    let node = parse_macro_input!(input as Node);

    TokenStream2::from(node).into()
}
