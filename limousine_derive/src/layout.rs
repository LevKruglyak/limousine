use crate::component::*;
use proc_macro2::{Ident, Span};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input, Arm, Token,
};

#[derive(Debug)]
pub struct IndexLayout {
    name: String,
    pub top: TopComponent,
    pub internal: Vec<InternalComponent>,
    pub base: BaseComponent,
}

impl IndexLayout {
    pub fn name(&self) -> Ident {
        Ident::new(&self.name, Span::mixed_site())
    }
}

impl Parse for IndexLayout {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let name: syn::Result<Ident> = input.parse();
        if name
            .map(|ident| ident.to_string() != "name".to_string())
            .unwrap_or(true)
        {
            return Err(syn::Error::new(span, "No name specified for the index!"));
        }

        let _: Token![:] = input.parse()?;
        let name: Ident = input.parse()?;
        let _: Token![,] = input.parse()?;

        let span = input.span();
        let layout: syn::Result<Ident> = input.parse();
        if layout
            .map(|ident| ident.to_string() != "layout".to_string())
            .unwrap_or(true)
        {
            return Err(syn::Error::new(span, "No layout specified for the index!"));
        }

        let _: Token![:] = input.parse()?;
        let content;
        bracketed!(content in input);

        let components: Vec<ParsedComponent> = content
            .parse_terminated(ParsedComponent::parse, Token![,])
            .map(|parsed| parsed.into_iter().collect())?;

        let first = components
            .first()
            .ok_or(syn::Error::new(Span::call_site(), "Empty layout!"))?;

        let top = TopComponent::from_general(first.component)
            .ok_or(syn::Error::new(first.span, "Invalid top component type!"))?;

        if components.len() == 1 {
            return Err(syn::Error::new(
                Span::call_site(),
                "No internal or base layers provided!",
            ));
        }

        let mut internal: Vec<InternalComponent> = Vec::new();
        for parsed in &components[1..components.len() - 1] {
            internal.push(InternalComponent::from_general(parsed.component).ok_or(
                syn::Error::new(first.span, "Invalid internal component type!"),
            )?);
        }

        let base = BaseComponent::from_general(components.last().unwrap().component)
            .ok_or(syn::Error::new(first.span, "Invalid base component type."))?;

        Ok(Self {
            name: name.to_string(),
            top,
            internal,
            base,
        })
    }
}
