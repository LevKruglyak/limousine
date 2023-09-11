use crate::util::Attribute;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Token,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Component {
    BTreeTop,
    BTree { fanout: usize, persist: bool },
    PGM { epsilon: usize },
}

impl Component {
    pub fn is_persisted(&self) -> bool {
        match self {
            Self::BTreeTop => false,
            Self::BTree { persist, .. } => *persist,
            _ => false,
        }
    }
}

pub struct ParsedComponent {
    pub span: Span,
    pub component: Component,
}

impl Parse for ParsedComponent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let ident: Ident = input.parse()?;
        let content;
        parenthesized!(content in input);

        // Get attributes
        let mut attrs = HashSet::new();
        for attr in content
            .parse_terminated(Attribute::parse, Token![,])
            .map(|parsed| parsed.into_iter())?
        {
            attrs.insert(attr);
        }

        let result = match ident.to_string().as_str() {
            "btree_top" => Ok(Self {
                span,
                component: Component::BTreeTop,
            }),
            "btree" => {
                let fanout = attrs
                    .get("fanout")
                    .expect("No fanout specified!")
                    .lit_int()
                    .expect("Fanout is not an integer!");

                let persist = attrs.get("persist").is_some();

                attrs.remove("fanout");
                attrs.remove("persist");

                Ok(Self {
                    span,
                    component: Component::BTree {
                        fanout: fanout.base10_parse()?,
                        persist,
                    },
                })
            }
            "pgm" => {
                let eps = attrs
                    .get("epsilon")
                    .expect("No epsilon specified!")
                    .lit_int()
                    .expect("Fanout is not an integer");

                attrs.remove("epsilon");

                Ok(Self {
                    span,
                    component: Component::PGM {
                        epsilon: eps.base10_parse()?,
                    },
                })
            }
            _ => Err(syn::Error::new(
                span,
                input.error("Invalid component type!"),
            )),
        };

        if !attrs.is_empty() {
            return Err(syn::Error::new(
                span,
                input.error("Invalid attribute specified!"),
            ));
        }

        result
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TopComponent {
    BTreeTop,
}

impl TopComponent {
    pub fn from_general(component: Component) -> Option<Self> {
        match component {
            Component::BTreeTop => Some(Self::BTreeTop),
            _ => None,
        }
    }

    pub fn to_tokens(&self, base: impl ToTokens) -> TokenStream {
        match self {
            &TopComponent::BTreeTop => {
                quote! { BTreeTopComponent<K, #base> }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InternalComponent {
    BTree { fanout: usize, persist: bool },
    PGM { epsilon: usize },
}

impl InternalComponent {
    pub fn from_general(component: Component) -> Option<Self> {
        match component {
            Component::BTree { fanout, persist } => Some(Self::BTree { fanout, persist }),
            Component::PGM { epsilon } => Some(Self::PGM { epsilon }),
            _ => None,
        }
    }

    pub fn to_tokens(&self, base: impl ToTokens) -> TokenStream {
        match self {
            &InternalComponent::BTree {
                fanout,
                persist: false,
            } => quote!(BTreeInternalComponent<K, #base, #fanout>).to_token_stream(),

            &InternalComponent::BTree {
                fanout,
                persist: true,
            } => quote!(BTreeInternalComponent<K, #base, #fanout>).to_token_stream(),

            &InternalComponent::PGM { epsilon } => {
                quote!(PGMInternalComponent<K, #base, #epsilon>).to_token_stream()
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BaseComponent {
    BTree { fanout: usize, persist: bool },
    PGM { epsilon: usize },
}

impl BaseComponent {
    pub fn from_general(component: Component) -> Option<Self> {
        match component {
            Component::BTree { fanout, persist } => Some(Self::BTree { fanout, persist }),
            Component::PGM { epsilon } => Some(Self::PGM { epsilon }),
            _ => None,
        }
    }

    pub fn to_tokens(&self) -> TokenStream {
        match self {
            &BaseComponent::BTree {
                fanout,
                persist: false,
            } => quote!(BTreeBaseComponent<K, V, #fanout>).to_token_stream(),

            &BaseComponent::BTree {
                fanout,
                persist: true,
            } => quote!(BTreeBaseComponent<K, V, #fanout>).to_token_stream(),

            &BaseComponent::PGM { epsilon } => quote!(PGMBaseComponent<K, V, #epsilon>),
        }
    }
}
