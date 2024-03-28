use crate::HybridLayout;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

mod disk;
mod memory;

pub fn create_implementation(name: Ident, layout: HybridLayout) -> proc_macro::TokenStream {
    let mod_name = proc_macro2::Ident::new(
        format!("__{}", name.to_string().to_lowercase()).as_str(),
        proc_macro2::Span::call_site(),
    );

    let (alias_body, alias) = create_type_aliases(&layout);

    let (index_body, index_fields) = if layout.is_persisted() {
        disk::create_index_struct(&name, &layout, &alias)
    } else {
        memory::create_index_struct(&name, &layout, &alias)
    };

    let index_impl = if layout.is_persisted() {
        disk::create_index_impl(&name, &layout, &alias, &index_fields)
    } else {
        memory::create_index_impl(&name, &layout, &alias, &index_fields)
    };

    eprintln!("{:#?}", alias);

    let mut implementation = proc_macro2::TokenStream::new();
    implementation.extend(quote! {
        pub mod #mod_name {
            use ::limousine_engine::private::*;

            #alias_body

            #index_body

            #index_impl
        }

        use #mod_name::#name;
    });

    implementation.into()
}

fn create_type_aliases(layout: &HybridLayout) -> (TokenStream, Vec<Ident>) {
    let address_alias: Vec<Ident> = (0..=layout.internal.len() + 1)
        .map(|i| Ident::new(format!("A{}", i).as_str(), Span::call_site()))
        .collect();

    let mut type_alias_body = proc_macro2::TokenStream::new();

    // Add body as the first component
    let alias = address_alias[0].clone();
    let body = layout.base.address_type();
    type_alias_body.extend(quote::quote! {
        type #alias = #body;
    });

    // Add internal components
    for (mut index, component) in layout.internal.iter().rev().enumerate() {
        index += 1;

        let alias = address_alias[index].clone();
        let body = component.address_type();
        type_alias_body.extend(quote! {
            type #alias = #body;
        });
    }

    let alias = address_alias.last().unwrap().clone();
    type_alias_body.extend(quote! { type #alias = (); });

    let type_alias: Vec<Ident> = (0..=layout.internal.len() + 1)
        .map(|i| Ident::new(format!("C{}", i).as_str(), Span::call_site()))
        .collect();

    // Add body as the first component
    let parent_address_alias = address_alias[1].clone();
    let alias = type_alias[0].clone();
    let body = layout.base.component_type(parent_address_alias);
    type_alias_body.extend(quote::quote! {
        type #alias<K, V> = #body;
    });

    // Add internal components
    for (mut index, component) in layout.internal.iter().rev().enumerate() {
        index += 1;

        let base_address_alias = address_alias[index - 1].clone();
        let parent_address_alias = address_alias[index + 1].clone();

        let body = component.component_type(base_address_alias, parent_address_alias);

        let alias = type_alias[index].clone();
        type_alias_body.extend(quote! {
            type #alias<K, V> = #body;
        });
    }

    // Add top component
    let index = layout.internal.len() + 1;

    let base_address_alias = address_alias[index - 1].clone();
    let body = layout.top.component_type(base_address_alias);

    let alias = type_alias[index].clone();
    type_alias_body.extend(quote! {
        type #alias<K, V> = #body;
    });

    (type_alias_body, type_alias)
}
