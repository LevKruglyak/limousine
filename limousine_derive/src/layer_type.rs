use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    LitInt,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LayerType {
    BTree { fanout: usize },
    PGM { epsilon: usize },
}

impl LayerType {
    pub fn to_base(&self) -> LayerBaseType {
        match self {
            &LayerType::BTree { .. } => LayerBaseType::BTree,
            &LayerType::PGM { .. } => LayerBaseType::PGM,
        }
    }
}

impl ToTokens for LayerType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token_stream = match self {
            &LayerType::BTree { fanout } => {
                quote!(::limousine_engine::private::BTreeLayer<K, #fanout>).to_token_stream()
            }
            &LayerType::PGM { epsilon } => {
                quote!(::limousine_engine::private::PGMLayer<K, #epsilon>).to_token_stream()
            }
        };

        token_stream.to_tokens(tokens);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LayerBaseType {
    BTree,
    PGM,
}

impl ToTokens for LayerBaseType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token_stream = match self {
            &LayerBaseType::BTree => {
                quote!(::limousine_engine::private::BTreeLayer).to_token_stream()
            }
            &LayerBaseType::PGM => quote!(::limousine_engine::private::PGMLayer).to_token_stream(),
        };

        token_stream.to_tokens(tokens);
    }
}

impl Parse for LayerType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "btree" => {
                let content;
                parenthesized!(content in input);
                let fanout: LitInt = content.parse()?;
                Ok(LayerType::BTree {
                    fanout: fanout.base10_parse()?,
                })
            }
            "pgm" => {
                let content;
                parenthesized!(content in input);
                let epsilon: LitInt = content.parse()?;
                Ok(LayerType::PGM {
                    epsilon: epsilon.base10_parse()?,
                })
            }
            _ => Err(syn::Error::new(
                span,
                input.error("expected 'btree()', 'pgm()'"),
            )),
        }
    }
}
