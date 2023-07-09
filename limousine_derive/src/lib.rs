//! This crate contains macros to materialize hybrid index designs as part of
//! the limousine engine.
//!
//! This crate should not be imported directly, (it will not work) rather the macros
//! should be accessed through the [`limousine_engine`](https://crates.io/crates/limousine_engine)
//! crate.

#![deny(missing_docs)]
use crate::layer_type::LayerType;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced, parse,
    parse::{Parse, ParseStream},
    parse_macro_input, Arm, Token,
};

mod layer_type;

/// Macro to materialize an immutable hybrid index structure. To materialize a hybrid index,
/// a name and a layout is required, for example:
///
/// ```ignore
/// // Note: this is just a syntax example, this macro needs to be called
/// // from the [`limousine_engine`](https://crates.io/crates/limousine_engine) crate.
///
/// create_immutable_hybrid_index! {
///     name: MyHybridIndex,
///     layout: {
///         0 | 1 => btree(32),
///         _ => pgm(8),
///     }
/// }
/// ```
/// The name is required to be some unique identifier which is not defined anywhere elsewhere in
/// the scope. The layout follows the Rust match expression syntax, where each arm follows the
/// format:
///
/// ```ignore
/// [usize match body] => [layer_type](param1, param2, ...)
/// ```
///
/// The supported layer types are:
///
/// 1. **btree(fanout: usize)**
/// 2. **pgm(epsilon: usize)**
///
/// The macro will generate a structure with the given name, alongside an implementation of the
/// `ImmutableIndex` trait.
#[proc_macro]
pub fn create_immutable_hybrid_index(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Generate documentation
    let input_string = input.to_string();
    let mut documentation = quote! {
        #[doc = "Immutable hybrid index materialized by layout:"]
        #[doc = ""]
    };

    for line in input_string.lines() {
        let line = format!(
            "```rust
        {{
        {line}
        }}
        ```"
        );
        documentation.extend(quote!(#[doc = #line]));
    }

    // Parse hybrid index description
    let desc = parse_macro_input!(input as HybridIndexDescription);
    let name = Ident::new(&desc.name, Span::mixed_site());

    let layer_name = Ident::new(
        &format!("__{}Layer", &desc.name).to_string(),
        Span::call_site(),
    );

    // Materialize layer type
    let mut variants = Vec::new();
    for (i, layer) in desc.layer_types.iter().enumerate() {
        let variant_name = format_ident!("L{}", i);
        variants.push(quote! { #variant_name(#layer) });
    }

    // Materialize implementation for layer type
    let mut len_arms = Vec::new();
    let mut search_arms = Vec::new();
    let mut key_iter_arms = Vec::new();

    for (i, _) in desc.layer_types.iter().enumerate() {
        let variant_name = format_ident!("L{}", i);

        len_arms.push(quote!(crate::#layer_name::#variant_name(_internal) => _internal.len()));
        search_arms.push(
            quote!(crate::#layer_name::#variant_name(_internal) => _internal.search(key, range)),
        );
        key_iter_arms.push(quote!(crate::#layer_name::#variant_name(_internal) => Box::new(_internal.nodes().iter().map(|x| *x.borrow()))));
    }

    let mut build_arms = Vec::new();
    let mut build_on_disk_arms = Vec::new();
    let mut load_arms = Vec::new();

    for (arm, index) in desc.layout.iter().cloned() {
        let layer_base = desc.layer_types[index].to_base();
        let layout_variant_name = format_ident!("L{}", index);

        let mut build_arm = arm.clone();
        build_arm.body = Box::new(
            syn::parse2(quote!(crate::#layer_name::#layout_variant_name(#layer_base::build(base))))
                .expect(""),
        );
        build_arms.push(build_arm.to_token_stream());

        let mut build_on_disk_arm = arm.clone();
        build_on_disk_arm.body = Box::new(
            syn::parse2(quote!(Ok(crate::#layer_name::#layout_variant_name(#layer_base::build_on_disk(base, path)?))))
                .expect(""),
        );
        build_on_disk_arms.push(build_on_disk_arm.to_token_stream());

        let mut load_arm = arm.clone();
        load_arm.body = Box::new(
            syn::parse2(
                quote!(Ok(crate::#layer_name::#layout_variant_name(#layer_base::load(path)?))),
            )
            .expect(""),
        );

        load_arms.push(load_arm.to_token_stream());
    }

    // Layer implementation
    let mut layer = quote! {
        pub enum #layer_name<K: ::limousine_engine::private::Key> {
            #(#variants),*
        }

        impl<K: ::limousine_engine::private::Key> ::limousine_engine::private::HybridLayer<K> for #layer_name<K> {
            fn len(&self) -> usize {
                use ::limousine_engine::private::NodeLayer;

                match self {
                    #(#len_arms,)*
                }
            }

            fn search(&self, key: &K, range: ::limousine_engine::private::ApproxPos) -> ::limousine_engine::private::ApproxPos {
                use ::limousine_engine::private::InternalLayer;

                match self {
                    #(#search_arms,)*
                }
            }

            fn build(layer: usize, base: impl ExactSizeIterator<Item = K>) -> Self {
                use ::limousine_engine::private::InternalLayerBuild;

                match layer {
                    #(#build_arms)*
                }
            }

            fn build_on_disk(
                layer: usize,
                base: impl ExactSizeIterator<Item = K>,
                path: impl AsRef<std::path::Path>,
            ) -> ::limousine_engine::private::Result<Self>
            where
                Self: Sized {
                use ::limousine_engine::private::InternalLayerBuild;

                match layer {
                    #(#build_on_disk_arms)*
                }
            }

            fn load(layer: usize, path: impl AsRef<std::path::Path>) -> ::limousine_engine::private::Result<Self>
            where
                Self: Sized {
                use ::limousine_engine::private::InternalLayerBuild;

                match layer {
                    #(#load_arms)*
                }
            }

            fn key_iter<'e>(&'e self) -> Box<dyn ExactSizeIterator<Item = K> + 'e> {
                use ::limousine_engine::private::NodeLayer;
                use std::borrow::Borrow;

                match self {
                    #(#key_iter_arms,)*
                }
            }
        }
    };

    // Index implementation
    let index = quote! {
        #documentation
        #[allow(unused_import)]
        pub struct #name<K: ::limousine_engine::private::Key, V: ::limousine_engine::private::Value>(::limousine_engine::private::HybridIndex<K, V, #layer_name<K>>);

        impl<K: ::limousine_engine::private::Key, V: ::limousine_engine::private::Value> ::limousine_engine::private::ImmutableIndex<K, V> for #name<K, V> {
            fn build_in_memory(base: impl ExactSizeIterator<Item = (K, V)>) -> Self {
                Self(::limousine_engine::private::HybridIndex::build_in_memory(base))
            }

            fn build_on_disk(
                base: impl ExactSizeIterator<Item = (K, V)>,
                path: impl AsRef<::std::path::Path>,
                threshold: usize,
            ) -> ::limousine_engine::private::Result<Self> {
                Ok(Self(::limousine_engine::private::HybridIndex::build_on_disk(base, path, threshold)?))
            }

            fn load(path: impl AsRef<::std::path::Path>, threshold: usize) -> ::limousine_engine::private::Result<Self> {
                Ok(Self(::limousine_engine::private::HybridIndex::load(path, threshold)?))
            }

            fn lookup(&self, key: &K) -> Option<V> {
                self.0.lookup(key)
            }

            fn range(&self, low: &K, high: &K) -> Self::RangeIterator<'_> {
                self.0.range(low, high)
            }

            type RangeIterator<'e> = ::limousine_engine::private::HybridIndexRangeIterator<'e, K, V>;
        }
    };

    layer.extend(index);
    layer.into()
}

struct HybridIndexDescription {
    name: String,
    layer_types: Vec<LayerType>,
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
            let body = arm.body.to_token_stream();
            let layer_type: LayerType = parse(body.into())?;

            if !layer_types.contains(&layer_type) {
                layer_types.push(layer_type.clone());
            }

            let layer_index = layer_types
                .iter()
                .enumerate()
                .find(|&(_, item)| *item == layer_type)
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
