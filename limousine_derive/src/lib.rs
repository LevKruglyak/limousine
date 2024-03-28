use proc_macro2::{Ident, TokenStream};
use syn::bracketed;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::{LitStr, Token};

#[proc_macro]
pub fn create_hybrid_index(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);

    codegen::create_implementation(input.name, input.layout)
}

macro_rules! bail {
    ($msg:expr) => {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            $msg,
        ))
    };
    ($span:expr, $msg:expr) => {
        return Err(syn::Error::new_spanned(&$span, $msg));
    };
    ($span:expr, $fmt:expr, $($args:tt)*) => {
        return Err(syn::Error::new_spanned(&$span, format!($fmt, $($args)*)));
    };
}

pub(crate) use bail;

mod codegen;
mod component;
mod layout;

use layout::HybridLayout;

struct MacroInput {
    name: Ident,
    layout: HybridLayout,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut path = None;
        let mut layout = None;

        // Parse the fields of the input struct
        while !input.is_empty() {
            let field_ident = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;
            let field = field_ident.to_string();

            // Decide how to parse the expression
            match field.as_str() {
                "name" => {
                    if name.is_some() {
                        bail!(field_ident, "`name` is already defined!");
                    }

                    let name_ident = input.parse::<Ident>()?;
                    name = Some(name_ident);
                }
                "path" => {
                    if path.is_some() {
                        bail!(field_ident, "`path` is already defined!");
                    }

                    let path_lit = input.parse::<LitStr>()?;
                    path = Some(path_lit);
                }
                "layout" => {
                    if layout.is_some() {
                        bail!(field_ident, "`layout` is already defined!");
                    }

                    let layout_buffer;
                    bracketed!(layout_buffer in input);
                    let layout_stream: TokenStream = layout_buffer.parse()?;
                    layout = Some(layout_stream);
                }
                field => {
                    bail!(field_ident, "No rule to process field `{}`!", field);
                }
            }

            // Ensure that only one layout is provided
            if path.is_some() && layout.is_some() {
                bail!(field_ident, "Cannot have both `layout` and `path` fields!");
            }

            if let Some(ref path) = path {
                let file_contents = std::fs::read_to_string(path.value());

                // Try reading from the file
                #[allow(clippy::needless_late_init)]
                let layout_contents;
                match file_contents {
                    Ok(contents) => {
                        layout_contents = contents;
                    }
                    Err(error) => {
                        bail!(path, error.to_string());
                    }
                }

                // Parse the contents
                layout = Some(syn::parse_str(layout_contents.as_str())?);
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let layout_stream: proc_macro::TokenStream;
        if let Some(layout) = layout {
            layout_stream = layout.into();
        } else {
            bail!("No `layout` or `path` specified!");
        }

        let name_ident;
        if let Some(name) = name {
            name_ident = name;
        } else {
            bail!("No `name` specified!")
        }

        let layout = syn::parse(layout_stream)?;
        eprintln!("{:?}", layout);

        Ok(Self {
            name: name_ident,
            layout,
        })
    }
}
