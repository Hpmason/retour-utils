use std::ops::Deref;

use proc_macro2::TokenStream;
use syn::{Ident, ItemMod, ItemFn, Item, spanned::Spanned, LitStr, FnArg, Attribute, Pat, Signature};

use crate::parse::HookAttributeArgs;


pub struct Module<'a> {
    pub original: &'a ItemMod,
    libary_module: &'a LitStr,
    functions: Vec<Function<'a>>,
    other_items: Vec<&'a Item>
}

pub struct Function<'a> {
    pub original: &'a ItemFn,
    other_attrs: Vec<&'a Attribute>,
    pub detour: Option<HookAttributeArgs>,
}

impl<'a> Module<'a> {
    pub fn from_syn(node: &'a ItemMod, module_name_meta: &'a LitStr) -> syn::Result<Self> {
        // eprintln!("{node:?}");
        let mut functions = Vec::new();
        let mut other_items = Vec::new();
        if let Some(content) = &node.content {
            for item in &content.1 {
                let Item::Fn(func) = item else {
                    other_items.push(item);
                    continue;
                };
                functions.push(Function::from_syn(func)?);
            }
        };

        Ok(Self {
            original: node,
            functions,
            other_items,
            libary_module: module_name_meta,
            
        })
    }

    pub fn get_fns(&self) -> Vec<&Function> {
        self.functions.iter().collect()
    }

    pub fn get_module_ident(&self) -> &Ident {
        &self.original.ident
    }
    pub fn get_normal_fns(&self) -> Vec<&'a Function> {
        self.functions.iter().filter(|func| !func.is_detour_fn()).collect()
    }

    pub fn get_detour_fns(&self) -> Vec<&'a Function> {
        self.functions.iter().filter(|func| func.is_detour_fn()).collect()
    }
    /// Returns a list of items other than functions that were originally in this module 
    /// (things that should not be modified by the macro)
    pub fn get_items(&self) -> &Vec<&'a Item> {
        &self.other_items
    }

    pub fn library_name(&self) -> &LitStr {
        &self.libary_module
    }

}


impl<'a> Function<'a> {
    pub fn from_syn(func: &'a ItemFn) -> syn::Result<Self> {
        let mut hook_data: Option<HookAttributeArgs> = None;
        let mut remaining_attrs = Vec::new();
        for attr in &func.attrs {
            if attr.path.is_ident("hook") {
                if let Some(existing_attr) = hook_data {
                    let mut duplicate_error = syn::Error::new(attr.span(), "Only 1 #[hook] can be applied to a function");
                    duplicate_error.combine(syn::Error::new(
                        existing_attr.span(), 
                        "Only 1 #[hook] can be applied to a function"
                    ));
                    return Err(duplicate_error);
                }
                // eprintln!("Meta: {meta:#?}");
                hook_data = Some(attr.parse_args()?);
            }
            else {
                remaining_attrs.push(attr);
            }
        }
        // Hook functions can't be unsafe
        if let Some(hook) = &hook_data {
            if let Some(unsafety) = func.sig.unsafety {
                match hook.unsafety {
                    Some(hook_unsafety) => {
                        let mut detour_sig_err = syn::Error::new(unsafety.span, "Hook functions can't be unsafe!\nYou've already marked the target function as unsafe. \nIf that's what you want, you can remove this `unsafe`");
                        detour_sig_err.combine(syn::Error::new(hook_unsafety.span(), "You marked the target function as unsafe here"));
                        return Err(detour_sig_err)
                    },
                    None => {
                        let mut detour_sig_err = syn::Error::new(unsafety.span, "Hook functions can't be unsafe!\nIf your target function is unsafe, add it in the #[hook] macro");
                        detour_sig_err.combine(syn::Error::new(hook.detour_name.span(), "Try adding `unsafe` before the detour name"));
                        return Err(detour_sig_err)
                    },
                }
            }
        }
        Ok(Self {
            original: func,
            other_attrs: remaining_attrs,
            detour: hook_data,
        })
    }

    pub fn is_detour_fn(&self) -> bool {
        self.detour.is_some()
    }

    pub fn get_hook_name(&self) -> Option<&Ident> {
        self.detour.as_ref()
            .and_then(|det| Some(&det.detour_name))
    }
    
    pub fn get_arg_names(&self) -> Result<Vec<&Pat>, syn::Error> {
        self.original.sig.inputs
            .iter()
            .map(|arg| {
                let FnArg::Typed(arg) = arg else {
                    return Err(syn::Error::new(arg.span(), ""));
                };
                match *arg.pat {
                    syn::Pat::Ident(_) => Ok(&*arg.pat),
                    syn::Pat::Path(_) => Ok(&*arg.pat),
                    _ => Err(syn::Error::new(arg.pat.span(), "Function argument is not a supported pattern")),
                }
            })
            .collect()
    }

    pub fn get_lookup_data_constructor(&self, library_name: &LitStr) -> Option<TokenStream> {
        self.detour
            .as_ref()
            .and_then(|det| Some(det.hook_info.get_lookup_data_new_fn(library_name)))
    }

    pub fn get_modified_fn_item(&self) -> ItemFn {

        let attrs = self.other_attrs.iter().map(|attr| attr.deref().clone()).collect();
        ItemFn {
            attrs,
            ..self.original.clone()
        }
    }
}

pub fn fn_type_from_sig(sig: &Signature) -> TokenStream {
    let input_types: Vec<TokenStream> = sig.inputs
        .iter()
        .map(|fn_arg| {
            let FnArg::Typed(arg) = fn_arg else {
                return syn::Error::new(fn_arg.span(), "")
                    .into_compile_error()
            };
            let ty = &arg.ty;
            quote::quote_spanned!{ty.span()=>
                #ty
            }
        })
        .collect();
    let output_type = &sig.output;
    let abi = &sig.abi;
    let unsafety = &sig.unsafety;
    
    quote::quote_spanned!{sig.span()=>
        #unsafety #abi fn(#(#input_types),*) #output_type
    }
}
