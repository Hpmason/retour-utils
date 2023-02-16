use proc_macro2::TokenStream;
use syn::{ItemMod, LitStr, Signature, Ident, spanned::Spanned};

use crate::{types::{Module, Function}, crate_refs::{parent_crate, retour_crate}};

pub fn expand(mod_block: &ItemMod, attribute_meta: &LitStr) -> Result<TokenStream, syn::Error> {
    
    let module = Module::from_syn(mod_block, attribute_meta)?;
    let mod_name = module.get_module_ident();
    let funcs: Vec<TokenStream> = expand_fns(&module.get_fns());
    let items = module.get_items();

    let detours_defs: Vec<TokenStream> = module.get_detour_fns().into_iter().map(expand_detour_def).collect::<Result<Vec<TokenStream>, syn::Error>>()?;

    let library_name = module.library_name();
    let init_detours_fn = expand_init_detours_fn(library_name, &module.get_fns());
    Ok(quote::quote! {
        mod #mod_name {
            /// Name of the lirbary being hooked
            const MODULE_NAME: &str = #library_name;
            #init_detours_fn
            #(#items)*
            #(#funcs)*
            #(#detours_defs)*
        }
    })
}

fn expand_fns(fns: &Vec<&Function>) -> Vec<TokenStream> {
    fns.iter()
        .map(|func| {
            let func_item = &func.get_modified_fn_item();
            quote::quote!{
                #func_item
            }
        })
        .collect()
}

fn expand_detour_def(detour_func: &Function) -> Result<TokenStream, syn::Error> {
    assert!(detour_func.is_detour_fn());
    let type_sig = detour_func.get_type_sig();
    let detour_name = detour_func.get_hook_name()
        .expect("Only called on detour functions");
    let vis = detour_func.get_hook_vis();
    
    let func_decl = Signature {
        ident: Ident::new("__ffi_detour", detour_func.original.sig.ident.span()),
        ..detour_func.original.sig.clone()
    };
    let arg_names = detour_func.get_arg_names()?;
    let detour_krate = retour_crate();
    Ok(quote::quote_spanned!{detour_name.span()=>
        #vis static #detour_name: ::#detour_krate::StaticDetour<#type_sig> = {
            #[inline(never)]
            #[allow(unused_unsafe)]
            #func_decl {
                (#detour_name.__detour())(#(#arg_names),*)
            }
            ::#detour_krate::StaticDetour::__new(__ffi_detour)
        };
    })
}

pub fn expand_init_detours_fn(library_name: &LitStr, detour_fns: &Vec<&Function>) -> TokenStream {
    let krate_name = parent_crate();
    let init_funcs: Vec<TokenStream> = detour_fns.iter()
        .filter(|f| f.is_detour_fn())
        .map(|&func| {
            expand_init_detour(func, library_name)
        })
        .collect();
    quote::quote!{
        pub unsafe fn init_detours() -> Result<(), #krate_name::Error> {
            #(#init_funcs;)*

            return Ok(())
        }
    }
}

pub fn expand_init_detour(detour_fn: &Function, library_name: &LitStr) -> TokenStream {
    let lookup_new_fn = detour_fn.get_lookup_data_constructor(library_name)
        .expect("only be called on detour fns");
    let detour_name = detour_fn.get_hook_name().unwrap();
    let orig_func_name = &detour_fn.original.sig.ident;
    let parent_krate = parent_crate();
    let detour_krate = retour_crate();
    quote::quote_spanned!{detour_fn.original.span()=>
        ::#parent_krate::init_detour(
            #lookup_new_fn,
            |addr| {
                #detour_name
                    .initialize(::#detour_krate::Function::from_ptr(addr), #orig_func_name)?
                    .enable()?;
                Ok(())
            }
        )?
    }
}