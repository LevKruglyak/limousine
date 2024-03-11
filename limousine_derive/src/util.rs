use std::borrow::Borrow;
use std::hash::Hash;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Lit, LitInt, Token};

pub struct Attribute {
    pub key: Ident,
    pub key_string: String,
    pub value: Option<Expr>,
}

impl Hash for Attribute {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}

impl Borrow<str> for Attribute {
    fn borrow(&self) -> &str {
        &self.key_string.as_str()
    }
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Attribute {}

impl Attribute {
    pub fn lit_int(&self) -> Option<LitInt> {
        if let Some(Expr::Lit(expr_lit)) = self.value.clone() {
            if let Lit::Int(lit_int) = expr_lit.lit {
                return Some(lit_int);
            }
        }

        None
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
