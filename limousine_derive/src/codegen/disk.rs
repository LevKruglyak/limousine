use crate::HybridLayout;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

pub fn create_index_struct(
    name: &Ident,
    _layout: &HybridLayout,
    alias: &[Ident],
) -> (TokenStream, Vec<Ident>) {
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
            store: GlobalStore,
        }
    };

    (body, fields)
}

pub fn create_index_impl(
    name: &Ident,
    layout: &HybridLayout,
    aliases: &[Ident],
    fields: &[Ident],
) -> TokenStream {
    let search_body = create_search_body(layout, aliases, fields);
    let insert_body = create_insert_body(layout, aliases, fields);
    let load_body = create_load_body(layout, aliases, fields);
    let build_body = create_build_body(layout, aliases, fields);

    let body = quote! {
        impl<K: Key, V: Value> PersistedIndex<K, V> for #name<K, V>
        where
            K: limousine_engine::private::Persisted,
            V: limousine_engine::private::Persisted,
        {
            fn search(&self, key: K) -> limousine_engine::Result<Option<V>> {
                #search_body
            }

            fn insert(&mut self, key: K, value: V) -> limousine_engine::Result<Option<V>> {
                #insert_body
            }

            fn load(path: impl AsRef<Path>) -> limousine_engine::Result<Self> {
                #load_body
            }

            fn build(iter: impl Iterator<Item = (K, V)>, path: impl AsRef<Path>) -> limousine_engine::Result<Self> {
                #build_body
            }
        }
    };

    body
}

fn create_search_body(layout: &HybridLayout, _aliases: &[Ident], fields: &[Ident]) -> TokenStream {
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

    search_body.extend(quote! { let #search = self.#field.search(&self.#next, key);});

    // Internal components
    for index in 1..=layout.internal.len() {
        let search = search_vars[index].clone();
        let prev_search = search_vars[index - 1].clone();
        let field = component_vars[index].clone();
        let next = component_vars[index + 1].clone();

        if layout.internal[index - 1].is_persisted() {
            search_body.extend(
                quote! { let #search = self.#field.search(&self.#next, #prev_search, key)?;},
            );
        } else {
            search_body.extend(
                quote! { let #search = self.#field.search(&self.#next, #prev_search, key);},
            );
        }
    }

    // Base component
    let index = layout.internal.len() + 1;
    let search = search_vars[index].clone();
    let prev_search = search_vars[index - 1].clone();
    let field = component_vars[index].clone();

    search_body.extend(quote! { let #search = self.#field.search(#prev_search, key)?;});
    search_body.extend(quote! { Ok(#search) });

    search_body
}

fn create_insert_body(layout: &HybridLayout, _aliases: &[Ident], fields: &[Ident]) -> TokenStream {
    let search_vars: Vec<Ident> = (0..=layout.internal.len() + 1)
        .rev()
        .map(|i| Ident::new(format!("s{}", i).as_str(), Span::call_site()))
        .collect();

    let component_vars: Vec<Ident> = fields.iter().cloned().rev().collect();
    let mut insert_body = TokenStream::new();

    // Top component
    let search = search_vars[0].clone();
    let field = component_vars[0].clone();
    let next = component_vars[1].clone();

    insert_body.extend(quote! { let #search = self.#field.search(&self.#next, key);});

    // Internal components
    for index in 1..=layout.internal.len() {
        let search = search_vars[index].clone();
        let prev_search = search_vars[index - 1].clone();
        let field = component_vars[index].clone();
        let next = component_vars[index + 1].clone();

        if layout.internal[index - 1].is_persisted() {
            insert_body.extend(
                quote! { let #search = self.#field.search(&self.#next, #prev_search, key)?;},
            );
        } else {
            insert_body.extend(
                quote! { let #search = self.#field.search(&self.#next, #prev_search, key);},
            );
        }
    }

    // Base component
    let index = layout.internal.len() + 1;
    let search = search_vars[index].clone();
    let prev_search = search_vars[index - 1].clone();
    let field = component_vars[index].clone();

    insert_body.extend(quote! { let #search = self.#field.search(#prev_search, key)?;});

    insert_body.extend(quote! { let result = s0; });

    // Insert stage
    let insert_vars: Vec<Ident> = (0..=layout.internal.len() + 1)
        .map(|i| Ident::new(format!("i{}", i).as_str(), Span::call_site()))
        .collect();

    let var = insert_vars[0].clone();
    let field = fields[0].clone();
    let search = search_vars[search_vars.len() - 2].clone();

    insert_body.extend(quote! {
        let #var;
        if let Some(x) = self.#field.insert(#search, key, value)? {
            #var = x;
        } else {
            return Ok(result);
        }
    });

    for index in 1..=layout.internal.len() {
        let var = insert_vars[index].clone();
        let prev_var = insert_vars[index - 1].clone();

        let field = fields[index].clone();
        let prev_field = fields[index - 1].clone();

        if layout.internal[index - 1].is_persisted() {
            insert_body.extend(quote! {
                let #var;
                if let Some(x) = self.#field.insert(&mut self.#prev_field, #prev_var)? {
                    #var = x;
                } else {
                    return Ok(result);
                }
            });
        } else {
            insert_body.extend(quote! {
                let #var;
                if let Some(x) = self.#field.insert(&mut self.#prev_field, #prev_var) {
                    #var = x;
                } else {
                    return Ok(result);
                }
            });
        }
    }

    let index = layout.internal.len() + 1;

    let var = insert_vars[index].clone();
    let prev_var = insert_vars[index - 1].clone();

    let field = fields[index].clone();
    let prev_field = fields[index - 1].clone();

    insert_body.extend(quote! {
        let #var = self.#field.insert(&mut self.#prev_field, #prev_var);
    });

    insert_body.extend(quote! { Ok(result) });
    insert_body
}

fn create_load_body(layout: &HybridLayout, aliases: &[Ident], fields: &[Ident]) -> TokenStream {
    eprintln!("layout: {:#?}\n", layout);

    let mut empty_body = TokenStream::new();

    // Add body as the first component
    let alias = aliases[0].clone();
    let var = fields[0].clone();

    empty_body.extend(quote! {
        // Load the store
        let mut store = GlobalStore::load(path)?;
    });

    // Base layer is guaranteed to be a disk component
    let alias_name = alias.to_string();
    empty_body.extend(quote! {
        // Load the store
        let mut #var = #alias::load(&mut store, #alias_name)?;
    });

    // Add internal components
    for index in 1..=layout.internal.len() {
        let alias = aliases[index].clone();
        let var = fields[index].clone();
        let prev_var = fields[index - 1].clone();

        let alias_name = alias.to_string();
        if layout.internal[index - 1].is_persisted() {
            empty_body.extend(quote! {
                let mut #var = #alias::load(&mut store, #alias_name)?;
            });
        } else {
            empty_body.extend(quote! {
                let mut #var = #alias::build(&mut #prev_var);
            });
        }
    }

    let index = layout.internal.len() + 1;
    let alias = aliases[index].clone();
    let var = fields[index].clone();
    let prev_var = fields[index - 1].clone();

    empty_body.extend(quote! {
        let mut #var = #alias::build(&mut #prev_var);
    });

    empty_body.extend(quote! {
        Ok(Self {
            #(#fields,)*
            store,
        })
    });

    empty_body
}

fn create_build_body(layout: &HybridLayout, aliases: &[Ident], fields: &[Ident]) -> TokenStream {
    let mut build_body = TokenStream::new();

    build_body.extend(quote! {
        // Load the store
        let mut store = GlobalStore::load(path)?;
    });

    // Add body as the first component
    let alias = aliases[0].clone();
    let var = fields[0].clone();

    let alias_name = alias.to_string();
    build_body.extend(quote! {
        let mut #var = #alias::build(&mut store, #alias_name, iter)?;
    });

    // Add internal components
    for index in 1..=layout.internal.len() {
        let alias = aliases[index].clone();
        let var = fields[index].clone();
        let prev_var = fields[index - 1].clone();

        let alias_name = alias.to_string();
        if layout.internal[index - 1].is_persisted() {
            build_body.extend(quote! {
                let mut #var = #alias::build(&mut store, #alias_name)?;
            });
        } else {
            build_body.extend(quote! {
                let mut #var = #alias::build(&mut #prev_var);
            });
        }
    }

    let index = layout.internal.len() + 1;
    let alias = aliases[index].clone();
    let var = fields[index].clone();
    let prev_var = fields[index - 1].clone();

    build_body.extend(quote! {
        let mut #var = #alias::build(&mut #prev_var);
    });

    build_body.extend(quote! {
        Ok(Self {
            #(#fields,)*
            store,
        })
    });

    build_body
}
