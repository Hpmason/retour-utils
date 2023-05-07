use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::{
    fold::Fold, token::Const, Expr, ExprLit, Generics, ItemConst, ItemFn,
    LitStr, Signature, Token,
    Visibility, spanned::Spanned, Item, FnArg, Pat,
};

use crate::{parse::HookAttributeArgs, helpers::{str_type_from_span, fn_type}, crate_refs};

#[derive(Debug)]
pub struct Detours {
    module_name: LitStr,
    detours: Vec<DetourInfo>,
}

impl Detours {
    pub fn new(module_name: LitStr) -> Self {
        Self {
            module_name,
            detours: Vec::new(),
        }
    }

    pub fn generate_detour_decls(&self) -> Vec<Item> {
        self.detours
            .iter()
            .map(|info| info.get_static_detour())
            .collect()
    }

    /// Returns the const expression containing the module name
    /// ```
    /// pub const MODULE_NAME: &str = "lua52.dll";
    /// ```
    pub fn get_module_name_decl(&self) -> Item {
        let module_name = &self.module_name;

        Item::Verbatim(quote_spanned!{self.module_name.span()=>
            pub const MODULE_NAME: &str = #module_name;
        })
    }
}



#[derive(Debug)]
pub struct DetourInfo {
    pub hook_attr: HookAttributeArgs,
    pub fn_sig: Signature,
}

impl DetourInfo {
    fn get_static_detour(&self) -> Item {
        let vis = self.hook_attr.vis.clone();
        
        let detour_krate = crate_refs::retour_crate();
        let detour_name = &self.hook_attr.detour_name;
        let fn_type_sig = fn_type(&self.fn_sig, &self.hook_attr);
        let target_fn_decl = self.target_fn_decl();
        let arg_names = self.get_arg_names().unwrap();

        Item::Verbatim(quote_spanned!{self.hook_attr.span()=>
            #[allow(non_upper_case_globals)]
            #vis static #detour_name: ::#detour_krate::StaticDetour<#fn_type_sig> = {
                #[inline(never)]
                #[allow(unused_unsafe)]
                #target_fn_decl {
                    #[allow(unused_unsafe)]
                    (#detour_name.__detour())(#(#arg_names),*)
                }
                ::#detour_krate::StaticDetour::__new(__ffi_detour)
            };
        })
    }

    fn target_fn_decl(&self) -> TokenStream {
        let input_types: Vec<TokenStream> = self.fn_sig.inputs
            .iter()
            .map(|fn_arg| {
                let FnArg::Typed(arg) = fn_arg else {
                    return syn::Error::new(fn_arg.span(), "A")
                        .into_compile_error()
                };
                quote::quote_spanned!{arg.span()=>
                    #arg
                }
            })
        .collect();
        let output_type = &self.fn_sig.output;
        let abi = &self.hook_attr.abi;
        let unsafety = &self.hook_attr.unsafety;
        
        quote::quote_spanned!{self.hook_attr.span()=>
            #unsafety #abi fn __ffi_detour(#(#input_types),*) #output_type
        }
    }

    fn get_arg_names(&self) -> Result<Vec<&Pat>, syn::Error> {
        self.fn_sig.inputs
        .iter()
        .map(|arg| {
            let FnArg::Typed(arg) = arg else {
                return Err(syn::Error::new(arg.span(), "jkhgakdfjhg"));
            };
            match *arg.pat {
                syn::Pat::Ident(_) => Ok(&*arg.pat),
                syn::Pat::Path(_) => Ok(&*arg.pat),
                _ => Err(syn::Error::new(arg.pat.span(), "Function argument is not a supported pattern")),
            }
        })
        .collect()
    }
}

impl Fold for Detours {
    fn fold_item_fn(&mut self, item_fn: syn::ItemFn) -> syn::ItemFn {
        let mut attrs = Vec::new();

        for attr in item_fn.attrs {
            if !attr.path().is_ident("hook") {
                attrs.push(attr);
                continue;
            }
            let Ok(hook_attrs) = attr.parse_args::<HookAttributeArgs>() else {
                continue;
            };
            self.detours.push(DetourInfo {
                hook_attr: hook_attrs,
                fn_sig: item_fn.sig.clone(),
            })
        }
        ItemFn { attrs, ..item_fn }
    }
}
