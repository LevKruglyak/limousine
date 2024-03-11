use crate::bail;
use crate::component::{BaseComponent, InternalComponent, ParsedComponent, TopComponent};
use syn::parse::Parse;
use syn::Token;

pub struct HybridLayout {
    pub top: TopComponent,
    pub internal: Vec<InternalComponent>,
    pub base: BaseComponent,
}

impl HybridLayout {
    pub fn is_persisted(&self) -> bool {
        self.internal
            .iter()
            .any(|component| component.is_persisted())
            || self.base.is_persisted()
    }
}

impl Parse for HybridLayout {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Get all of the components in order
        let components: Vec<ParsedComponent> = input
            .parse_terminated(ParsedComponent::parse, Token![,])
            .map(|parsed| parsed.into_iter().collect())?;

        let mut in_persisted_region: bool = false;

        // Parse the top component
        let top;
        if let Some(first) = components.first() {
            in_persisted_region |= first.is_persisted();

            if let Some(top_component) = TopComponent::try_new(first.into()) {
                top = top_component;
            } else {
                bail!(first.ident(), "Invalid top component type!");
            }
        } else {
            bail!("Empty layout!");
        }

        // Parse the internal components
        if components.len() == 1 {
            bail!("No internal or base layers specified!");
        }

        let mut internal = Vec::new();
        for parsed in &components[1..] {
            if !parsed.is_persisted() && in_persisted_region {
                bail!(
                    parsed.ident(),
                    "Cannot have an in-memory component below a persisted component!"
                );
            }

            in_persisted_region |= parsed.is_persisted();

            if let Some(internal_component) = InternalComponent::try_new(parsed.into()) {
                internal.push(internal_component);
            } else {
                bail!(parsed.ident(), "Invalid internal component type!");
            }
        }

        // Parse base components
        let base;
        if let Some(base_component) = BaseComponent::try_new(components.last().unwrap().into()) {
            base = base_component;
        } else {
            bail!("Invalid base component type!")
        }

        Ok(Self {
            top,
            internal,
            base,
        })
    }
}
