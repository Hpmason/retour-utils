use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{fold::Fold, spanned::Spanned, Item, ItemFn, LitStr, Signature};

use crate::{
    crate_refs,
    helpers::{fn_arg_names, fn_type},
    parse::HookAttributeArgs,
};

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

        Item::Verbatim(quote_spanned! {self.module_name.span()=>
            #[allow(unused)]
            pub const MODULE_NAME: &str = #module_name;
        })
    }

    pub fn generate_init_detours(&self) -> Item {
        let krate_name = crate_refs::parent_crate();
        let init_funcs: Vec<Item> = self
            .detours
            .iter()
            .map(|func| func.generate_detour_init(&self.module_name))
            .collect();
        Item::Verbatim(quote::quote! {
            pub unsafe fn init_detours() -> Result<(), #krate_name::Error> {
                #(#init_funcs;)*

                Ok(())
            }
        })
    }
}

pub struct DetourInfo {
    pub hook_attr: HookAttributeArgs,
    pub fn_sig: Signature,
}

impl DetourInfo {
    fn get_static_detour(&self) -> Item {
        let vis = self.hook_attr.vis.clone();

        let detour_krate = crate_refs::retour_crate();
        let detour_name: &proc_macro2::Ident = &self.hook_attr.detour_name;
        let fn_type_sig = fn_type(&self.fn_sig, &self.hook_attr);
        let target_fn_decl = self.target_fn_decl();
        let arg_names = fn_arg_names(&self.fn_sig).unwrap();

        Item::Verbatim(quote_spanned! {self.hook_attr.span()=>
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
        let input_types = self.fn_sig.inputs.iter();
        // output includes the `->` in the return type
        let output_type = &self.fn_sig.output;
        let abi = &self.hook_attr.abi;
        let unsafety = &self.hook_attr.unsafety;

        quote::quote_spanned! {self.hook_attr.span()=>
            #unsafety #abi fn __ffi_detour(#(#input_types),*) #output_type
        }
    }

    fn generate_detour_init(&self, module_name: &LitStr) -> Item {
        let lookup_new_fn = (self.hook_attr.hook_info).get_lookup_data_new_fn(module_name);
        let detour_name = &self.hook_attr.detour_name;
        let orig_func_name = &self.fn_sig.ident;
        let parent_krate = crate_refs::parent_crate();
        let detour_krate = crate_refs::retour_crate();
        Item::Verbatim(quote_spanned! {self.hook_attr.span()=>
            ::#parent_krate::init_detour(
                #lookup_new_fn,
                |addr| {
                    #detour_name
                        .initialize(::#detour_krate::Function::from_ptr(addr), #orig_func_name)?
                        .enable()?;
                    Ok(())
                }
            )?
        })
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
