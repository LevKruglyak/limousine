#![allow(unused, unused_imports, dead_code)]
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::parse_str;
use syn::punctuated::Punctuated;
use syn::token::Plus;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    parse_macro_input, Arm, Expr, Pat, Token,
};
use syn::{parenthesized, parse, LitInt};

#[proc_macro]
pub fn materialize_index(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let desc = parse_macro_input!(input as HybridIndexDescription);
    // eprintln!("{desc:#?}");

    let name = Ident::new(&desc.name, Span::mixed_site());

    let layer_name = Ident::new(
        &format!("__{}Layer", &desc.name).to_string(),
        Span::call_site(),
    );

    let mut variants = Vec::new();
    for (i, layer) in desc.layer_types.iter().enumerate() {
        let variant_name = format_ident!("L{}", i);
        let (layer_name, param) = match layer {
            LayerBase::BTree { fanout } => ("BTreeLayer", fanout),
            LayerBase::PGM { epsilon } => ("PGMLayer", epsilon),
        };
        let layer_type =
            parse_str::<TokenStream>(&format!("::clearned_core::{}<K, {}>", layer_name, param))
                .expect("");

        variants.push(quote! { #variant_name(#layer_type) });
    }

    // let mut match_expression = Vec::new();
    // for (arm, layer) in desc.layout.iter() {
    //     let variant_name = format_ident!("L{}", layer);
    //     let (layer_name, param) = match layer {
    //         LayerBase::BTree { fanout } => ("BTreeLayer", fanout),
    //         LayerBase::PGM { epsilon } => ("PGMLayer", epsilon),
    //     };
    //     let layer_type =
    //         parse_str::<TokenStream>(&format!("::clearned_core::{}<K, {}>", layer_name, param))
    //             .expect("");
    //
    //     variants.push(quote! { #variant_name(#layer_type) });
    // }

    let mut len_arms = Vec::new();
    let mut search_arms = Vec::new();
    let mut key_iter_arms = Vec::new();

    for (i, layer) in desc.layer_types.iter().enumerate() {
        let variant_name = format_ident!("L{}", i);
        len_arms.push(
            parse_str::<TokenStream>(&format!(
                "crate::{}::{}(_internal) => _internal.len(),",
                layer_name, variant_name
            ))
            .expect(""),
        );

        search_arms.push(
            parse_str::<TokenStream>(&format!(
                "crate::{}::{}(_internal) => _internal.search(key, range),",
                layer_name, variant_name
            ))
            .expect(""),
        );

        key_iter_arms.push(
            parse_str::<TokenStream>(&format!(
                "crate::{}::{}(_internal) => Box::new(_internal.nodes().iter().map(|x| *x.borrow())),",
                layer_name, variant_name
            ))
            .expect(""),
        );
    }

    let mut build_arms = Vec::new();
    let mut build_on_disk_arms = Vec::new();
    let mut load_arms = Vec::new();

    for (arm, index) in desc.layout.iter().cloned() {
        let layout_type = desc.layer_types[index];
        let layout_variant_name = format_ident!("L{}", index);

        let name = match layout_type {
            LayerBase::BTree { fanout } => "BTreeLayer",
            LayerBase::PGM { epsilon } => "PGMLayer",
        };
        let layer_type = parse_str::<TokenStream>(&format!("::clearned_core::{}", name)).expect("");

        let mut build_arm = arm.clone();
        build_arm.body = Box::new(
            syn::parse2(quote!(crate::#layer_name::#layout_variant_name(#layer_type::build(base))))
                .expect(""),
        );
        build_arms.push(build_arm.to_token_stream());

        let mut build_on_disk_arm = arm.clone();
        build_on_disk_arm.body = Box::new(
            syn::parse2(quote!(Ok(crate::#layer_name::#layout_variant_name(#layer_type::build_on_disk(base, path)?))))
                .expect(""),
        );
        build_on_disk_arms.push(build_on_disk_arm.to_token_stream());

        let mut load_arm = arm.clone();
        load_arm.body = Box::new(
            syn::parse2(
                quote!(Ok(crate::#layer_name::#layout_variant_name(#layer_type::load(path)?))),
            )
            .expect(""),
        );
        load_arms.push(load_arm.to_token_stream());
    }

    let mut layer = quote! {
        pub enum #layer_name<K: ::clearned_core::Key> {
            #(#variants),*
        }

        impl<K: ::clearned_core::Key> ::clearned_core::HybridLayer<K> for #layer_name<K> {
            fn len(&self) -> usize {
                use ::clearned_core::NodeLayer;

                match self {
                    #(#len_arms),*
                }
            }

            fn search(&self, key: &K, range: ::clearned_core::ApproxPos) -> ::clearned_core::ApproxPos {
                use ::clearned_core::InternalLayer;

                match self {
                    #(#search_arms),*
                }
            }

            fn build(layer: usize, base: impl ExactSizeIterator<Item = K>) -> Self {
                use ::clearned_core::InternalLayerBuild;

                match layer {
                    #(#build_arms)*
                }
            }

            fn build_on_disk(
                layer: usize,
                base: impl ExactSizeIterator<Item = K>,
                path: impl AsRef<std::path::Path>,
            ) -> ::clearned_core::Result<Self>
            where
                Self: Sized {
                use ::clearned_core::InternalLayerBuild;

                match layer {
                    #(#build_on_disk_arms)*
                }
            }

            fn load(layer: usize, path: impl AsRef<std::path::Path>) -> ::clearned_core::Result<Self>
            where
                Self: Sized {
                use ::clearned_core::InternalLayerBuild;

                match layer {
                    #(#load_arms)*
                }
            }

            fn key_iter<'e>(&'e self) -> Box<dyn ExactSizeIterator<Item = K> + 'e> {
                use ::clearned_core::NodeLayer;
                use std::borrow::Borrow;

                match self {
                    #(#key_iter_arms),*
                }
            }
        }
    };

    eprintln!("{}", layer.to_string());

    let index = quote! {
        #[allow(unused_import)]
        use ::clearned_core::ImmutableIndex;

        pub struct #name<K: ::clearned_core::Key, V: ::clearned_core::Value>(::clearned_core::HybridIndex<K, V, #layer_name<K>>);

        impl<K: ::clearned_core::Key, V: ::clearned_core::Value> ::clearned_core::ImmutableIndex<K, V> for #name<K, V> {
            fn build_in_memory(base: impl ExactSizeIterator<Item = (K, V)>) -> Self {
                Self(::clearned_core::HybridIndex::build_in_memory(base))
            }

            fn build_on_disk(
                base: impl ExactSizeIterator<Item = (K, V)>,
                path: impl AsRef<::std::path::Path>,
                threshold: usize,
            ) -> ::clearned_core::Result<Self> {
                Ok(Self(::clearned_core::HybridIndex::build_on_disk(base, path, threshold)?))
            }

            fn load(path: impl AsRef<::std::path::Path>, threshold: usize) -> ::clearned_core::Result<Self> {
                Ok(Self(::clearned_core::HybridIndex::load(path, threshold)?))
            }

            fn lookup(&self, key: &K) -> Option<V> {
                self.0.lookup(key)
            }

            fn range(&self, low: &K, high: &K) -> Self::RangeIterator<'_> {
                self.0.range(low, high)
            }

            type RangeIterator<'e> = ::clearned_core::HybridIndexRangeIterator<'e, K, V>;
        }
    };

    layer.extend(index);
    layer.into()
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum LayerBase {
    BTree { fanout: usize },
    PGM { epsilon: usize },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum LayerModifier {
    Persist,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum LayerComponent {
    Base(LayerBase),
    Modifier(LayerModifier),
}

impl Parse for LayerComponent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let lookahead = input.lookahead1();
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "btree" => {
                let content;
                parenthesized!(content in input);
                let fanout: LitInt = content.parse()?;
                Ok(LayerComponent::Base(LayerBase::BTree {
                    fanout: fanout.base10_parse()?,
                }))
            }
            "pgm" => {
                let content;
                parenthesized!(content in input);
                let epsilon: LitInt = content.parse()?;
                Ok(LayerComponent::Base(LayerBase::PGM {
                    epsilon: epsilon.base10_parse()?,
                }))
            }
            "persist" => Ok(LayerComponent::Modifier(LayerModifier::Persist)),
            _ => Err(syn::Error::new(
                span,
                input.error("expected 'btree()', 'pgm()', or 'persist'"),
            )),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct LayerTypePreDescription {
    description: Vec<LayerComponent>,
}

impl ToTokens for LayerTypePreDescription {
    fn to_tokens(&self, tokens: &mut TokenStream) {}
}

impl Parse for LayerTypePreDescription {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content = Punctuated::<LayerComponent, Plus>::parse_terminated(input)?;

        Ok(LayerTypePreDescription {
            description: content.into_iter().collect(),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct LayerTypeDescription {
    base: LayerBase,
    modifier: Option<LayerModifier>,
}

impl Parse for LayerTypeDescription {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: fix span issues
        let span = input.span();
        let mut content: LayerTypePreDescription = input.parse()?;

        let base = content
            .description
            .iter()
            .cloned()
            .find(|&x| matches!(x, LayerComponent::Base(x)));

        if base.is_none() {
            return Err(syn::Error::new(
                span,
                input.error("expected 'btree()', 'pgm()', or 'persist'"),
            ));
        }
        content.description.retain(|x| x != &base.unwrap());

        // Messy, but whatever for now
        let base = match base {
            Some(LayerComponent::Base(base)) => base,
            _ => unreachable!(),
        };

        let mut modifier = None;
        for component in content.description.iter().cloned() {
            match component {
                LayerComponent::Base(base) => {
                    return Err(syn::Error::new(
                        span,
                        input.error("multiple bases detected!"),
                    ));
                }
                LayerComponent::Modifier(new_modifier) => {
                    if modifier.is_none() {
                        modifier = Some(new_modifier);
                    } else {
                        return Err(syn::Error::new(
                            span,
                            input.error("multiple modifier detected!"),
                        ));
                    }
                }
            }
        }

        Ok(LayerTypeDescription {
            base: base.clone(),
            modifier,
        })
    }
}

struct HybridIndexDescription {
    name: String,
    layer_types: Vec<LayerBase>,
    layout: Vec<(Arm, usize)>,
}

impl Parse for HybridIndexDescription {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        assert!(name.to_string() == "name".to_string(), "missing name!");

        let _: Token![:] = input.parse()?;
        let index_name: Ident = input.parse()?;
        let _: Token![,] = input.parse()?;

        let layout: Ident = input.parse()?;
        assert!(
            layout.to_string() == "layout".to_string(),
            "missing layout!"
        );
        let _: Token![:] = input.parse()?;
        let content;
        braced!(content in input);
        let mut layout = Vec::new();
        let mut layer_types = Vec::new();

        while !content.is_empty() {
            let arm: Arm = content.parse()?;
            let _: Option<Token![,]> = content.parse()?;
            eprintln!("{:#?}", arm.body.to_token_stream().to_string());
            let body = arm.body.to_token_stream();
            let layer_type: LayerTypeDescription = parse(body.into())?;

            if !layer_types.contains(&layer_type.base) {
                layer_types.push(layer_type.base.clone());
            }

            let layer_index = layer_types
                .iter()
                .enumerate()
                .find(|&(_, item)| *item == layer_type.base)
                .unwrap()
                .0;

            layout.push((arm, layer_index));
        }

        let _: Option<Token![,]> = content.parse()?;

        Ok(Self {
            name: index_name.to_string(),
            layer_types,
            layout,
        })
    }
}
