use crate::bail;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Expr, Lit, LitInt, Token,
};

#[derive(Clone)]
pub enum Component {
    BTreeTop,
    BTree { fanout: usize, persist: bool },
}

pub struct ParsedComponent {
    ident: Ident,
    component: Component,
}

impl From<&ParsedComponent> for Component {
    fn from(value: &ParsedComponent) -> Self {
        value.component.clone()
    }
}

impl ParsedComponent {
    pub fn is_persisted(&self) -> bool {
        match self.component {
            Component::BTree { persist, .. } => persist,
            _ => false,
        }
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}

impl Parse for ParsedComponent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        let attributes;
        parenthesized!(attributes in input);

        let mut attributes: Attributes = attributes.parse()?;

        let component = match ident.to_string().as_str() {
            "btree_top" => Component::BTreeTop,
            "btree" => {
                let fanout = attributes.try_get_integer(&ident, "fanout")?;
                let persist = attributes.try_get_bool("persist")?;

                let fanout = if fanout >= 2 {
                    fanout as usize
                } else {
                    bail!(ident, "Specified fanout is less than 2!");
                };

                Component::BTree { fanout, persist }
            }
            _ => {
                bail!(ident, "Unknown component `{}`!", ident.to_string());
            }
        };

        Ok(Self { ident, component })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TopComponent {
    BTreeTop,
}

impl TopComponent {
    pub fn try_new(component: Component) -> Option<Self> {
        match component {
            Component::BTreeTop => Some(Self::BTreeTop),
            _ => None,
        }
    }

    pub fn component_type(&self, base_address: impl ToTokens) -> TokenStream {
        match *self {
            TopComponent::BTreeTop => {
                quote! { BTreeTopComponent<K, V, #base_address> }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InternalComponent {
    BTree { fanout: usize, persist: bool },
}

impl InternalComponent {
    pub fn try_new(component: Component) -> Option<Self> {
        match component {
            Component::BTree { fanout, persist } => Some(Self::BTree { fanout, persist }),
            _ => None,
        }
    }

    pub fn component_type(
        &self,
        base_address: impl ToTokens,
        parent_address: impl ToTokens,
    ) -> TokenStream {
        match *self {
            InternalComponent::BTree {
                fanout,
                persist: false,
            } => quote!(BTreeInternalComponent<K, V, #fanout, #base_address, #parent_address>)
                .to_token_stream(),

            InternalComponent::BTree {
                fanout,
                persist: true,
            } => quote!(BTreeInternalComponent<K, V, #fanout, #base_address, #parent_address>)
                .to_token_stream(),
        }
    }

    pub fn address_type(&self) -> TokenStream {
        match *self {
            InternalComponent::BTree { persist: false, .. } => {
                quote!(BTreeInternalAddress).to_token_stream()
            }

            InternalComponent::BTree { persist: true, .. } => {
                quote!(BTreeInternalAddress).to_token_stream()
            }
        }
    }

    pub fn is_persisted(&self) -> bool {
        match *self {
            InternalComponent::BTree { persist, .. } => persist,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BaseComponent {
    BTree { fanout: usize, persist: bool },
}

impl BaseComponent {
    pub fn try_new(component: Component) -> Option<Self> {
        match component {
            Component::BTree { fanout, persist } => Some(Self::BTree { fanout, persist }),
            _ => None,
        }
    }

    pub fn component_type(&self, base_address: impl ToTokens) -> TokenStream {
        match *self {
            BaseComponent::BTree {
                fanout,
                persist: false,
            } => quote!(BTreeBaseComponent<K, V, #fanout, #base_address>).to_token_stream(),

            BaseComponent::BTree {
                fanout,
                persist: true,
            } => quote!(BTreeBaseComponent<K, V, #fanout, #base_address>).to_token_stream(),
        }
    }

    pub fn address_type(&self) -> TokenStream {
        match *self {
            BaseComponent::BTree { persist: false, .. } => {
                quote!(BTreeBaseAddress).to_token_stream()
            }

            BaseComponent::BTree { persist: true, .. } => {
                quote!(BTreeBaseAddress).to_token_stream()
            }
        }
    }

    pub fn is_persisted(&self) -> bool {
        match *self {
            BaseComponent::BTree { persist, .. } => persist,
        }
    }
}

use std::borrow::Borrow;
use std::hash::Hash;

pub struct Attributes {
    attrs: HashSet<Attribute>,
}

impl Attributes {
    fn try_get_integer(&mut self, ident: &Ident, name: &str) -> syn::Result<i32> {
        if let Some(attr) = self.attrs.take(name) {
            if let Some(value) = attr.try_get_integer() {
                return value.base10_parse();
            }

            bail!(attr.key(), "Failed to parse integer attribute `{}`!", name);
        }

        bail!(ident, "Could not find required attribute `{}`!", name);
    }

    fn try_get_bool(&mut self, name: &str) -> syn::Result<bool> {
        if let Some(attr) = self.attrs.take(name) {
            if let Some(value) = attr.try_get_bool() {
                return Ok(value);
            }

            bail!(attr.key(), "Failed to parse boolean attribute `{}`!", name);
        }

        Ok(false)
    }
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = HashSet::new();
        for attr in input
            .parse_terminated(Attribute::parse, Token![,])
            .map(|parsed| parsed.into_iter())?
        {
            attrs.insert(attr);
        }

        Ok(Self { attrs })
    }
}

pub struct Attribute {
    key: Ident,
    key_string: String,
    value: Option<Expr>,
}

impl Hash for Attribute {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.to_string().hash(state)
    }
}

impl Borrow<str> for Attribute {
    fn borrow(&self) -> &str {
        self.key_string.as_str()
    }
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Attribute {}

impl Attribute {
    // Try parsing the attribute as an integer
    pub fn try_get_integer(&self) -> Option<LitInt> {
        if let Some(Expr::Lit(expr)) = self.value.clone() {
            if let Lit::Int(integer) = expr.lit {
                return Some(integer);
            }
        }

        None
    }

    // Try parsing the attribute as a boolean
    pub fn try_get_bool(&self) -> Option<bool> {
        if self.value.is_none() {
            return Some(true);
        }

        None
    }

    pub fn key(&self) -> &Ident {
        &self.key
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        let equal: Option<Token![=]> = input.parse()?;

        if equal.is_some() {
            let value: Expr = input.parse()?;

            return Ok(Attribute {
                key_string: key.to_string(),
                key,
                value: Some(value),
            });
        }

        Ok(Attribute {
            key_string: key.to_string(),
            key,
            value: None,
        })
    }
}
