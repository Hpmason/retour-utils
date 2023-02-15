use proc_macro2::{TokenStream, Ident};
use quote::{spanned::Spanned, ToTokens};
use syn::{NestedMeta, ItemMod, FnArg, Signature, Pat};

use crate::types::{Module, StaticDetour};



pub fn expand(mod_block: &ItemMod, attribute_meta: &NestedMeta) -> Result<TokenStream, syn::Error> {
    
    let module = Module::from_syn(mod_block, attribute_meta)?;
    let mod_name = module.module_ident();   
    let normal_funcs: Vec<&syn::ItemFn> = module.get_normal_fns().iter().map(|func| func.original).collect();
    let items = module.get_items();
    // if args.is_empty() {
    //     return Err(syn::Error::new(mod_block.ident.span(), "must provide library name"))
    // }
    let library_name = module.library_name();

    let static_detours: Vec<&StaticDetour> = module.get_detour_fns()
        .iter()
        .map(|func| func.get_static_detour().as_ref().unwrap())
        .collect();
    let static_detour_defs = expand_detour_definitions(&static_detours)?;
    let init_detours_fn = expand_init_detour_fn(&module, &static_detours)?;
    
    Ok(quote::quote! {
        mod #mod_name {
            /// Name of the lirbary being hooked
            const MODULE_NAME: &str = #library_name;
            #(#items)*
            #(#normal_funcs)*
            #static_detour_defs
            #init_detours_fn
        }
    })
}


fn expand_init_detour_fn(module: &Module, static_detours: &[&StaticDetour]) -> Result<TokenStream, syn::Error> {
    let init_detours: Vec<TokenStream> = static_detours.iter()
        .map(|detour| {
            let original_func = detour.original_fn_ident();
            let ident = detour.get_ident();
            let hook_data = detour.get_hook_data();
            let hook_data_new_fn = hook_data.get_new_fn(module.library_name());
            // let sig = detour.get_detour_sig();
            quote::quote!{
                ::detour_lib::init_detour_with_offset(
                    #hook_data_new_fn,
                    |addr| {
                        unsafe {
                            #ident.initialize(::retour::Function::from_ptr(addr), #original_func)?;
                            #ident.enable()?;
                            Ok(())
                        }
                    }
                )?;
            }
        }).collect();
    Ok(quote::quote! {
        pub unsafe fn init_detours() -> Result<(), ::retour::Error> {
            #(#init_detours)*
        }
    })
}

fn expand_detour_definitions(static_detours: &[&StaticDetour]) -> Result<TokenStream, syn::Error> {
    let statements: Vec<TokenStream> = static_detours
        .iter()
        .map(|detour| {
            let ident = detour.get_ident();
            let target_sig = detour.get_detour_sig();

            let fn_type = {
                let input_types: Vec<TokenStream> = target_sig.inputs.iter()
                    .map(|fn_arg| {
                        let FnArg::Typed(arg) = fn_arg else {
                            return syn::Error::new(fn_arg.__span(), "macro does not support bare self params")
                                .into_compile_error()
                        };
                        arg.ty.to_token_stream()
                    })
                    .collect::<Vec<TokenStream>>();
                    let output_type = &target_sig.output;
                quote::quote! {
                    fn(#(#input_types),*) #output_type
                }
            };
            let inner_sig = Signature {
                ident: Ident::new("__ffi_detour", target_sig.__span()),
                ..target_sig.clone()
            };
            let inner_arg_names: Vec<TokenStream> = inner_sig.inputs.iter()
                .map(|fn_arg| {
                    let FnArg::Typed(arg) = fn_arg else {
                        return syn::Error::new(fn_arg.__span(), "macro does not support bare self params")
                            .into_compile_error()
                    };
                    let Pat::Ident(pat_ident) = arg.pat.as_ref() else {
                        return syn::Error::new(arg.pat.__span(), "macro does not support bare self params")
                            .into_compile_error()
                    };
                    pat_ident.ident.to_token_stream()
                })
                .collect();
            eprintln!("inputs: {}", fn_type.to_token_stream().to_string());
            quote::quote! {
                static #ident: StaticDetour<#fn_type> = {
                    #inner_sig {
                        (#ident.__detour()).call(#(#inner_arg_names),*)
                    }
                    ::retour::StaticDetour::__new(__ffi_detour)
                };
            }
        })
        .collect();
    Ok(quote::quote! {
        #(#statements)*
    })
}