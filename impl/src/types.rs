use std::num::ParseIntError;

use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};
use syn::{Ident, ItemMod, ItemFn, Attribute, FnArg, Item, NestedMeta, spanned::Spanned, Lit, LitStr, Meta, Signature, punctuated::Punctuated, token::Comma};


pub struct Module<'a> {
    original: &'a ItemMod,
    libary_module: &'a LitStr,
    functions: Vec<Function<'a>>,
    other_items: Vec<&'a Item>
}

pub struct Function<'a> {
    pub original: &'a ItemFn,
    attrs: Vec<Attribute>,
    args: Vec<&'a FnArg>,
    detour: Option<StaticDetour<'a>>,
}

pub struct HookData<'a> {
    original: &'a Attribute,
    offset: Option<usize>,
    symbol: Option<String>,
}

pub struct StaticDetour<'a> {
    attribute: &'a Attribute,
    func: &'a ItemFn,
    hook_data: HookData<'a>,
    hook_sig: Signature,
}

impl<'a> Module<'a> {
    pub fn from_syn(node: &'a ItemMod, module_name_meta: &'a NestedMeta) -> syn::Result<Self> {
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
        let NestedMeta::Lit(Lit::Str(module_name)) = module_name_meta else {
            return Err(syn::Error::new(module_name_meta.span(), "Module name must be a string literal. It looks something like this: #[hook_module(\"<dll_name>\")]"));
        };

        Ok(Self {
            original: node,
            functions,
            other_items,
            libary_module: module_name,
            
        })
    }

    pub fn get_normal_fns(&self) -> Vec<&'a Function> {
        self.functions.iter().filter(|func| !func.is_detour_fn()).collect()
    }

    pub fn get_detour_fns(&self) -> Vec<&'a Function> {
        self.functions.iter().filter(|func| func.is_detour_fn()).collect()
    }

    pub fn get_items(&self) -> &Vec<&'a Item> {
        &self.other_items
    }

    pub fn module_ident(&self) -> &Ident {
        &self.original.ident
    }
    pub fn library_name(&self) -> &LitStr {
        &self.libary_module
    }
}


impl<'a> Function<'a> {
    pub fn from_syn(func: &'a ItemFn) -> syn::Result<Self> {
        let attrs = Vec::new();
        let fn_args: Vec<&'a FnArg> = func.sig.inputs.iter().map(|a| a).collect();

        let mut hook_data: Option<HookData> = None;
        for attr in &func.attrs {
            let meta = attr.parse_meta()?;
            let Some(path) = meta.path().get_ident() else {
                continue;
            };
            if path == "offset" {
                if let Some(existing_attr) = hook_data {
                    let mut duplicate_error = syn::Error::new(path.span(), "Only 1 #[offset] can be applied to a function");
                    duplicate_error.combine(syn::Error::new(
                        existing_attr.original.path.span(), 
                        "Only 1 #[offset] can be applied to a function"
                    ));
                    return Err(duplicate_error);
                }
                // eprintln!("Meta: {meta:#?}");
                hook_data = Some(get_offset_from_meta(attr, meta)?);
            }
        }
        let detour = if let Some(hook_data) = hook_data {
            Some(StaticDetour::new(func, hook_data)?)
        }
        else {
            None
        };
        Ok(Self {
            original: func,
            attrs,
            args: fn_args,
            detour,
        })
    }

    fn is_detour_fn(&self) -> bool {
        self.detour.is_some()
    }

    pub fn get_static_detour(&self) -> &Option<StaticDetour> {
        &self.detour
    }
}


impl<'a> HookData<'a> {
    fn from_offset(attribute: &'a Attribute, offset: usize) -> syn::Result<Self> {
        Ok(Self {
            original: attribute,
            offset: Some(offset),
            symbol: None,

        })
    }

    pub fn get_new_fn(&self, module: &'a LitStr) -> TokenStream {
        match (&self.offset, &self.symbol) {
            (Some(offset), None) => {
                quote::quote_spanned!{self.original.span()=>
                    ::detour_lib::LookupData::from_offset(#module, #offset)
                }
            },
            (None, Some(symbol)) =>{
                quote::quote_spanned!{self.original.span()=>
                    ::detour_lib::LookupData::from_symbol(#module, #symbol)
                }
            },
            _ => unreachable!(),
        }
    }
}

impl<'a> StaticDetour<'a> {
    fn new(func: &'a ItemFn, hook_data: HookData<'a>) -> Result<Self, syn::Error> {
        Ok(Self {
            attribute: hook_data.original,
            func: func,
            hook_data: hook_data,
            hook_sig: Self::get_target_sig(&func.sig)?,
        })
    }
    pub fn get_ident(&self) -> Ident {
        format_ident!("Dt{}", self.func.sig.ident)
    }
    pub fn get_hook_data(&self) -> &HookData {
        &self.hook_data
    }

    pub fn original_fn_ident(&self) -> &Ident {
        &self.func.sig.ident
    }

    pub fn get_detour_sig(&self) -> &Signature {
        &self.hook_sig
    }

    fn get_target_sig(original_sig: &'a Signature) -> Result<Signature, syn::Error> {
        eprintln!("{}", original_sig.to_token_stream().to_string());
        let inputs: Punctuated<FnArg, Comma> = {
            let mut iter = original_sig.inputs.clone().into_pairs();
            iter.next().ok_or(syn::Error::new(original_sig.paren_token.span, "The first parameter must be the detour"))?;
            iter.collect()
        };
        Ok(Signature {
            inputs,
            ..original_sig.clone()
        })
    }
}

fn parse_literal_offset(lit_val: &str) -> Result<usize, ParseIntError>{
    let without_prefix = lit_val
        .trim_start_matches("0x");
    without_prefix
        .parse()
        .or_else(|_| {
            usize::from_str_radix(without_prefix, 16)
        })
}

fn get_offset_from_meta<'a>(attr: &'a Attribute, meta: Meta) -> Result<HookData<'a>, syn::Error> {
    match &meta {
        // #[offset]
        syn::Meta::Path(path) => return Err(
            syn::Error::new(path.get_ident().unwrap().span(), "#[offset] attribute must contain a value. Try #[offset(xDEAD_BEEF)]")
        ),
        // #[offset(...)]
        syn::Meta::List(li) =>  {
            // Either #[offset()] or #[offset(.., .. ...)]
            if li.nested.len() != 1 {
                return Err(syn::Error::new(meta.span(), "#[offset()] only takes 1 argument, the an integer representing the offset"));
            }
            let nested_meta = li.nested.first().expect("already checked len");
            match nested_meta {
                NestedMeta::Lit(lit) => match lit {
                    syn::Lit::Int(i) => {
                        let offset = parse_literal_offset(&i.to_string())
                            .map_err(|_| syn::Error::new(i.span(), "Offset must be a valid integer!"))?;
                        Ok(HookData::from_offset(&attr, offset)?)
                    },
                    syn::Lit::Float(_) => todo!("error on floats"),
                    syn::Lit::Verbatim(_) => todo!("idk what verbatim means"),
                    _ => Err(
                        syn::Error::new(lit.span(), "Offset must be a literal integer!")
                    ),
                },
                NestedMeta::Meta(meta) => Err(syn::Error::new(meta.span(), "offset must be an integer!")),
            }
        },
        // #[offset = ...]
        syn::Meta::NameValue(nv) => match &nv.lit {
            syn::Lit::Int(i) => {
                let offset = parse_literal_offset(&i.to_string())
                    .map_err(|_| syn::Error::new(i.span(), "Offset must be a valid integer!"))?;
                Ok(HookData::from_offset(&attr, offset)?)
            },
            _ => return Err(
                syn::Error::new(nv.lit.span(), "Offset must be a literal integer!")
            ),
        },
    }
}


