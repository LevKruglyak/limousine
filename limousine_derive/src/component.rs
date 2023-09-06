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
    StdBTree,
    BTree { fanout: usize, persist: bool },
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

        match ident.to_string().as_str() {
            "std_btree" => Ok(Self {
                span,
                component: Component::StdBTree,
            }),
            "btree" => {
                let fanout = attrs
                    .get("fanout")
                    .expect("No fanout specified!")
                    .lit_int()
                    .expect("Fanout is not an integer!");

                let persist = attrs.get("persist").is_some();

                Ok(Self {
                    span,
                    component: Component::BTree {
                        fanout: fanout.base10_parse()?,
                        persist,
                    },
                })
            }
            _ => Err(syn::Error::new(
                span,
                input.error("Valid components: std_btree, btree"),
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TopComponent {
    StdBTree,
}

impl TopComponent {
    pub fn from_general(component: Component) -> Option<Self> {
        match component {
            Component::StdBTree => Some(Self::StdBTree),
            _ => None,
        }
    }

    pub fn to_tokens(&self, base: impl ToTokens) -> TokenStream {
        match self {
            &TopComponent::StdBTree => {
                quote! { BTreeTopComponent<K, #base> }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InternalComponent {
    BTree { fanout: usize, persist: bool },
}

impl InternalComponent {
    pub fn from_general(component: Component) -> Option<Self> {
        match component {
            Component::BTree { fanout, persist } => Some(Self::BTree { fanout, persist }),
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
                fanout: _,
                persist: true,
            } => unimplemented!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BaseComponent {
    BTree { fanout: usize, persist: bool },
}

impl BaseComponent {
    pub fn from_general(component: Component) -> Option<Self> {
        match component {
            Component::BTree { fanout, persist } => Some(Self::BTree { fanout, persist }),
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
                fanout: _,
                persist: true,
            } => unimplemented!(),
        }
    }
}
