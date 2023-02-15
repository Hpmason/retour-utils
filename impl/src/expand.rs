use proc_macro2::{TokenStream, Ident};
use syn::{NestedMeta, ItemMod};

use crate::types::Module;



pub fn expand(mod_block: &ItemMod, attribute_meta: &NestedMeta) -> Result<TokenStream, syn::Error> {
    
    let module = Module::from_syn(mod_block, attribute_meta)?;
    let mod_name = module.module_ident();   
    // if args.is_empty() {
    //     return Err(syn::Error::new(mod_block.ident.span(), "must provide library name"))
    // }
    Ok(quote::quote! {
        mod #mod_name {
            
        }
    })
}