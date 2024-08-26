use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_quote, GenericArgument, GenericParam, Ident, Index, ItemStruct, Type};

enum Key {
    Number(Ident, Ident),
    String,
    Boolean,
    Binary,
    Map(Box<[Key; 2]>),
    Vec(Box<Key>),
    Option(Box<Key>),
    Custom,
}

fn parse_ty(ty: &Type) -> Option<Key> {
    match ty {
        Type::Path(path) => {
            let segment = &path.path.segments[0];
            let ty = segment.ident.to_string();

            match ty.as_str() {
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
                    let ty = Ident::new(&ty.to_uppercase(), Span::call_site());

                    Some(Key::Number(segment.ident.clone(), ty))
                }
                "String" => Some(Key::String),
                "bool" => Some(Key::Boolean),
                "HashMap" => match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        if let (Some(GenericArgument::Type(k)), Some(GenericArgument::Type(v))) =
                            (args.args.first(), args.args.last())
                        {
                            Some(Key::Map(Box::new([parse_ty(k)?, parse_ty(v)?])))
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                "Vec" => match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        if let Some(GenericArgument::Type(t)) = args.args.first() {
                            let ty = parse_ty(t)?;

                            if let Key::Number(type_name, _) = &ty {
                                if type_name == "u8" {
                                    Some(Key::Binary)
                                } else {
                                    Some(Key::Vec(Box::new(ty)))
                                }
                            } else {
                                Some(Key::Vec(Box::new(ty)))
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                "Option" => match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        if let Some(GenericArgument::Type(t)) = args.args.first() {
                            Some(Key::Option(Box::new(parse_ty(t)?)))
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => Some(Key::Custom),
            }
        }
        _ => None,
    }
}

fn basic_ty(indexer: &TokenStream2, key: &Key, is_ref: bool) -> Option<TokenStream2> {
    match (key, is_ref) {
        (Key::Number(_, ty), false) => {
            Some(quote! { Some(crate::muzui::language::program::Value::#ty(#indexer)) })
        }
        (Key::Number(_, ty), true) => {
            Some(quote! { Some(crate::muzui::language::program::Value::#ty(*#indexer)) })
        }
        (Key::String, _) => {
            Some(quote! { Some(crate::muzui::language::program::Value::String(#indexer.clone())) })
        }
        (Key::Binary, _) => Some(
            quote! { Some(crate::muzui::language::program::Value::BinaryData(#indexer.clone())) },
        ),
        (Key::Boolean, false) => {
            Some(quote! { Some(crate::muzui::language::program::Value::Boolean(#indexer)) })
        }
        (Key::Boolean, true) => {
            Some(quote! { Some(crate::muzui::language::program::Value::Boolean(*#indexer)) })
        }
        (Key::Custom, _) => Some(quote! { #indexer.index(keys) }),
        _ => None,
    }
}

#[proc_macro_derive(Indexable)]
pub fn derive_indexable(item: TokenStream) -> TokenStream {
    let mut item = match syn::parse::<ItemStruct>(item) {
        Ok(value) => value,
        Err(error) => return error.into_compile_error().into(),
    };

    let name = item.ident;

    for generic in item.generics.params.iter_mut() {
        match generic {
            GenericParam::Type(param) => {
                param
                    .bounds
                    .push(parse_quote! { crate::muzui::language::program::Indexable });
            }
            _ => continue,
        }
    }

    let params = item
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(param) => Some(&param.ident),
            _ => None,
        })
        .collect::<Vec<_>>();

    let params = if params.is_empty() {
        quote! {}
    } else {
        quote! {
            <#(#params),*>
        }
    };

    let generics = item.generics;

    let mut string_fields = Vec::new();
    let mut number_fields = Vec::new();

    for (index, field) in item.fields.into_iter().enumerate() {
        let (pattern, indexer) = if let Some(name) = &field.ident {
            let name_str = name.to_string();

            (quote! { #name_str }, quote! { self.#name })
        } else {
            let index = Index::from(index);

            (
                quote! {
                    crate::muzui::language::program::Index::Number(#index)
                },
                quote! { self.#index },
            )
        };

        let value = parse_ty(&field.ty).and_then(|ty|
            basic_ty(&indexer, &ty, false)
                .map(|ty| quote! { #pattern => #ty, })
                .or_else(|| match ty {
                    Key::Map(value) => match value.as_ref() {
                        [Key::Number(k, _), v] => {
                            let v = basic_ty(&quote! { value }, v, true).unwrap_or_else(|| quote! { value.index(keys) });

                            Some(quote! {
                                #pattern => match keys.remove(0) {
                                    crate::muzui::language::program::Index::Number(value) => #indexer.get(&(value as #k)).and_then(|value| {
                                        #v
                                    }),
                                    _ => None,
                                }
                            })
                        }
                        [Key::String, v] => {
                            let v = basic_ty(&quote! { value }, v, true).unwrap_or_else(|| quote! { value.index(keys) });

                            Some(quote! {
                                #pattern => match keys.remove(0) {
                                    crate::muzui::language::program::Index::String(value) => #indexer.get(&value).and_then(|value| {
                                        #v
                                    }),
                                    _ => None,
                                },
                            })
                        }
                        _ => None
                    }
                    Key::Vec(value) => {
                        let ty = basic_ty(&quote! { value }, value.as_ref(), true).unwrap_or_else(|| quote! { value.index(keys) });

                        Some(quote! {
                            #pattern => match keys.remove(0) {
                                crate::muzui::language::program::Index::Number(value) => #indexer.get(value).and_then(|value| {
                                    #ty
                                }),
                                _ => None,
                            }
                        })
                    },
                    Key::Option(value) => {
                        let ty = basic_ty(&quote! { value }, value.as_ref(), true).unwrap_or_else(|| quote! { value.index(keys) });

                        Some(quote! {
                            #pattern => #indexer.as_ref().and_then(|value| {
                                #ty
                            }),
                            _ => None,
                        })
                    },
                    _ => None
                })
        ).unwrap_or_else(|| quote! { #pattern => #indexer.index(keys), });

        if field.ident.is_some() {
            string_fields.push(value);
        } else {
            number_fields.push(value);
        }
    }

    let a = quote! {
        impl #generics crate::muzui::language::program::Indexable for #name #params {
            fn index(&self, mut keys: Vec<crate::muzui::language::program::Index>) -> ::core::option::Option<crate::muzui::language::program::Value> {
                if keys.is_empty() {
                    return None;
                }

                match keys.remove(0) {
                    crate::muzui::language::program::Index::String(value) => match value.as_str() {
                        #(#string_fields)*
                        _ => None,
                    }
                    #(#number_fields)*
                    _ => None,
                }
            }
        }
    };

    a.into()
}
