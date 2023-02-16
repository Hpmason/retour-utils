use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::Ident;



pub fn retour_crate() -> Ident {
    let found_crate = crate_name("retour").expect("retour is present in `Cargo.toml`");

    match found_crate {
        FoundCrate::Itself => {
            Ident::new("retour", Span::call_site())
        },
        FoundCrate::Name(name) => {
            Ident::new(&name, Span::call_site())
        }
    }
}

pub fn parent_crate() -> Ident {
    let found_crate = crate_name("retour-utils").expect("detour-lib is present in `Cargo.toml`");

    match found_crate {
        FoundCrate::Itself => {
            Ident::new("retour_utils", Span::call_site())
        },
        FoundCrate::Name(name) => {
            Ident::new(&name.replace("-", "_"), Span::call_site())
        }
    }
}