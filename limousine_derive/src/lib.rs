//! This crate contains macros to materialize hybrid index designs as part of
//! the limousine engine.
//!
//! This crate should not be imported directly, (it will not work) rather the macros
//! should be accessed through the [`limousine_engine`](https://crates.io/crates/limousine_engine)
//! crate.

#![deny(missing_docs)]
mod component;
mod layout;
mod util;

use layout::IndexLayout;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// Creates an hybrid index structure.
///
/// To use this macro, provide a name for the generated structure and a layout. The layout defines
/// a list of layer types, arranged from the highest to the lowest layer.
///
/// # Example
/// ```rust,ignore
/// // Note: this is illustrative syntax. Ensure the macro is invoked from within
/// // the [`limousine_engine`](https://crates.io/crates/limousine_engine) crate.
///
/// create_hybrid_index! {
///     name: MyHybridIndex,
///     layout: {
///         btree_top(),                          // optimized top layer
///         pgm(fanout = 4),
///         pgm(fanout = 4),
///         btree(fanout = 32),                
///         btree(fanout = 32, persist),          // base layer (stored on disk)
///     }
/// }
/// ```
///
/// # Supported Top Component Types
/// - **btree_top()**
///
/// # Supported Internal/Base Component Types
/// - **btree(fanout = usize, persist?)**
/// - **pgm(epsilon = usize)**
///
/// Note: Not all layouts are valid. For example, placing a disk layer over an in-memory layer
/// will raise an error. The macro enforces these rules and generates a structure with the
/// given name, as well as an implementation of the `Index` trait.
#[proc_macro]
pub fn create_hybrid_index(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse hybrid index description
    let layout = syn::parse_macro_input!(input as IndexLayout);
    let name = layout.name();

    let mod_name = proc_macro2::Ident::new(
        format!("__implmentation_{}", name.to_string().to_lowercase()).as_str(),
        proc_macro2::Span::call_site(),
    );

    let (alias_body, alias) = create_type_aliases(&layout);
    let (index_body, index_fields) = create_index_struct(&layout, &alias);
    let index_impl = create_index_impl(&layout, &alias, &index_fields);

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

    // panic!("{}", implementation.to_string());
    implementation.into()
}

fn create_type_aliases(layout: &IndexLayout) -> (TokenStream, Vec<Ident>) {
    let address_alias: Vec<Ident> = (0..=layout.internal.len() + 1)
        .map(|i| Ident::new(format!("A{}", i).as_str(), Span::call_site()))
        .collect();

    let mut type_alias_body = proc_macro2::TokenStream::new();

    // Add body as the first component
    let alias = address_alias[0].clone();
    let body = layout.base.address();
    type_alias_body.extend(quote::quote! {
        type #alias = #body;
    });

    // Add internal components
    for (mut index, component) in layout.internal.iter().rev().enumerate() {
        index += 1;

        let alias = address_alias[index].clone();
        let body = component.address();
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
    let body = layout.base.as_tokens(parent_address_alias);
    type_alias_body.extend(quote::quote! {
        type #alias<K, V> = #body;
    });

    // Add internal components
    for (mut index, component) in layout.internal.iter().rev().enumerate() {
        index += 1;

        let base_address_alias = address_alias[index - 1].clone();
        let parent_address_alias = address_alias[index + 1].clone();

        let body = component.as_tokens(base_address_alias, parent_address_alias);

        let alias = type_alias[index].clone();
        type_alias_body.extend(quote! {
            type #alias<K, V> = #body;
        });
    }

    // Add top component
    let index = layout.internal.len() + 1;

    let base_address_alias = address_alias[index - 1].clone();
    let body = layout.top.as_tokens(base_address_alias);

    let alias = type_alias[index].clone();
    type_alias_body.extend(quote! {
        type #alias<K, V> = #body;
    });

    (type_alias_body, type_alias)
}

fn create_index_struct(layout: &IndexLayout, alias: &[Ident]) -> (TokenStream, Vec<Ident>) {
    let name = layout.name();

    // Create fields
    let mut fields = Vec::new();
    for component in alias.iter() {
        fields.push(Ident::new(
            component.to_string().to_lowercase().as_str(),
            Span::call_site(),
        ));
    }

    // Create field definitions
    let mut field_bodies = Vec::new();
    for (index, component) in alias.iter().enumerate() {
        let field = fields[index].clone();

        // TODO: remove the `pub` field, this is for debugging purposes
        field_bodies.push(quote! {
            pub #field: #component<K, V>,
        });
    }

    let body = quote! {
        pub struct #name<K: Key, V: Value> {
            #(#field_bodies)*
        }
    };

    (body, fields)
}

fn create_search_body(layout: &IndexLayout, _aliases: &[Ident], fields: &[Ident]) -> TokenStream {
    let search_vars: Vec<Ident> = (0..=layout.internal.len() + 1)
        .rev()
        .map(|i| Ident::new(format!("s{}", i).as_str(), Span::call_site()))
        .collect();

    let component_vars: Vec<Ident> = fields.iter().cloned().rev().collect();
    let mut search_body = TokenStream::new();

    // Top component
    let search = search_vars[0].clone();
    let field = component_vars[0].clone();
    let next = component_vars[1].clone();

    search_body.extend(quote! { let #search = self.#field.search(&self.#next, &key);});

    // Internal components
    for index in 1..=layout.internal.len() {
        let search = search_vars[index].clone();
        let prev_search = search_vars[index - 1].clone();
        let field = component_vars[index].clone();
        let next = component_vars[index + 1].clone();

        search_body.extend(quote! { let #search = self.#field.search(&self.#next, #prev_search, &key);});
    }

    // Base component
    let index = layout.internal.len() + 1;
    let search = search_vars[index].clone();
    let prev_search = search_vars[index - 1].clone();
    let field = component_vars[index].clone();

    search_body.extend(quote! { let #search = self.#field.search(#prev_search, &key);});
    search_body.extend(quote! { #search });

    search_body
}

fn create_insert_body(layout: &IndexLayout, _aliases: &[Ident], fields: &[Ident]) -> TokenStream {
    let search_vars: Vec<Ident> = (0..=layout.internal.len() + 1)
        .rev()
        .map(|i| Ident::new(format!("s{}", i).as_str(), Span::call_site()))
        .collect();

    let component_vars: Vec<Ident> = fields.iter().cloned().rev().collect();
    let mut search_body = TokenStream::new();

    // Top component
    let search = search_vars[0].clone();
    let field = component_vars[0].clone();
    let next = component_vars[1].clone();

    search_body.extend(quote! { let #search = self.#field.search(&self.#next, &key);});

    // Internal components
    for index in 1..=layout.internal.len() {
        let search = search_vars[index].clone();
        let prev_search = search_vars[index - 1].clone();
        let field = component_vars[index].clone();
        let next = component_vars[index + 1].clone();

        search_body.extend(quote! { let #search = self.#field.search(&self.#next, #prev_search, &key);});
    }

    // Base component
    let index = layout.internal.len() + 1;
    let search = search_vars[index].clone();
    let prev_search = search_vars[index - 1].clone();
    let field = component_vars[index].clone();

    search_body.extend(quote! { let #search = self.#field.search(#prev_search, &key);});

    search_body.extend(quote! { let result = s0.copied(); });

    // Insert stage
    let insert_vars: Vec<Ident> = (0..=layout.internal.len() + 1)
        .map(|i| Ident::new(format!("i{}", i).as_str(), Span::call_site()))
        .collect();

    let var = insert_vars[0].clone();
    let field = fields[0].clone();
    let search = search_vars[search_vars.len() - 2].clone();

    search_body.extend(quote! {
        let #var;
        if let Some(x) = self.#field.insert(#search, key, value) {
            #var = x;
        } else {
            return result;
        }
    });

    for index in 1..=layout.internal.len() {
        let var = insert_vars[index].clone();
        let prev_var = insert_vars[index - 1].clone();

        let field = fields[index].clone();
        let prev_field = fields[index - 1].clone();

        search_body.extend(quote! {
            let #var;
            if let Some(x) = self.#field.insert(&mut self.#prev_field, #prev_var) {
                #var = x;
            } else {
                return result;
            }
        });
    }

    let index = layout.internal.len() + 1;

    let var = insert_vars[index].clone();
    let prev_var = insert_vars[index - 1].clone();

    let field = fields[index].clone();
    let prev_field = fields[index - 1].clone();

    search_body.extend(quote! {
        let #var = self.#field.insert(&mut self.#prev_field, #prev_var);
    });

    search_body.extend(quote! { result });
    search_body
}

fn create_empty_body(layout: &IndexLayout, aliases: &[Ident], fields: &[Ident]) -> TokenStream {
    let mut empty_body = TokenStream::new();

    // Add body as the first component
    let alias = aliases[0].clone();
    let var = fields[0].clone();

    empty_body.extend(quote! {
        let mut #var = #alias::empty();
    });

    // Add internal components
    for index in 1..=layout.internal.len() {
        let alias = aliases[index].clone();
        let var = fields[index].clone();
        let prev_var = fields[index - 1].clone();

        empty_body.extend(quote! {
            let mut #var = #alias::build(&mut #prev_var);
        });
    }

    let index = layout.internal.len() + 1;
    let alias = aliases[index].clone();
    let var = fields[index].clone();
    let prev_var = fields[index - 1].clone();

    empty_body.extend(quote! {
        let mut #var = #alias::build(&mut #prev_var);
    });

    empty_body.extend(quote! {
        Self {
            #(#fields,)*
        }
    });

    empty_body
}

fn create_build_body(layout: &IndexLayout, aliases: &[Ident], fields: &[Ident]) -> TokenStream {
    let mut build_body = TokenStream::new();

    // Add body as the first component
    let alias = aliases[0].clone();
    let var = fields[0].clone();

    build_body.extend(quote! {
        let mut #var = #alias::build(iter);
    });

    // Add internal components
    for index in 1..=layout.internal.len() {
        let alias = aliases[index].clone();
        let var = fields[index].clone();
        let prev_var = fields[index - 1].clone();

        build_body.extend(quote! {
            let mut #var = #alias::build(&mut #prev_var);
        });
    }

    let index = layout.internal.len() + 1;
    let alias = aliases[index].clone();
    let var = fields[index].clone();
    let prev_var = fields[index - 1].clone();

    build_body.extend(quote! {
        let mut #var = #alias::build(&mut #prev_var);
    });

    build_body.extend(quote! {
        Self {
            #(#fields,)*
        }
    });

    build_body
}

fn create_index_impl(layout: &IndexLayout, aliases: &[Ident], fields: &[Ident]) -> TokenStream {
    let name = layout.name();

    let search_body = create_search_body(layout, aliases, fields);
    let insert_body = create_insert_body(layout, aliases, fields);
    let empty_body = create_empty_body(layout, aliases, fields);
    let build_body = create_build_body(layout, aliases, fields);

    let body = quote! {
        impl<K: Key, V: Value> #name<K, V> {
            pub fn search(&self, key: &K) -> Option<&V> {
                #search_body
            }

            pub fn insert(&mut self, key: K, value: V) -> Option<V> {
                #insert_body
            }

            pub fn empty() -> Self {
                #empty_body
            }

            pub fn build(iter: impl Iterator<Item = Entry<K, V>>) -> Self {
                #build_body
            }
        }
    };

    body
}
