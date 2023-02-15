use std::num::ParseIntError;

use syn::{Ident, ItemMod, ItemFn, Attribute, FnArg, Item, NestedMeta, spanned::Spanned, AttributeArgs, Lit, LitStr};


pub struct Module<'a> {
    original: &'a ItemMod,
    libary_module: &'a LitStr,
    functions: Vec<Function<'a>>,
    other_items: Vec<&'a Item>
}

pub struct Function<'a> {
    original: &'a ItemFn,
    attrs: &'a Vec<Attribute>,
    args: Vec<&'a FnArg>,
    hook_data: Option<HookData<'a>>,
}

pub struct HookData<'a> {
    original: &'a Attribute,
    offset: Option<usize>,
    symbol: Option<String>,

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

    pub fn module_ident(&self) -> &Ident {
        &self.original.ident
    }
}


impl<'a> Function<'a> {
    pub fn from_syn(func: &'a ItemFn) -> syn::Result<Self> {
        let attrs = &func.attrs;
        let fn_args: Vec<&'a FnArg> = func.sig.inputs.iter().map(|a| a).collect();

        let mut hook_data: Option<HookData> = None;
        for attr in attrs {
            let meta = attr.parse_meta()?;
            let Some(path) = meta.path().get_ident() else {
                continue;
            };
            if path == "offset" {
                if let Some(existing_attr) = hook_data {
                    let mut duplicate_error = syn::Error::new(path.span(), "Only 1 #[offset] can be applied to a function");
                    duplicate_error.combine(syn::Error::new(
                        existing_attr.original.path.get_ident().unwrap().span(), 
                        "Only 1 #[offset] can be applied to a function"
                    ));
                    return Err(duplicate_error);
                }
                // eprintln!("Meta: {meta:#?}");
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
                                    hook_data = Some(HookData::from_offset(attr, offset)?);
                                },
                                syn::Lit::Float(_) => todo!("error on floats"),
                                syn::Lit::Verbatim(_) => todo!("idk what verbatim means"),
                                _ => return Err(
                                    syn::Error::new(lit.span(), "Offset must be a literal integer!")
                                ),
                            },
                            NestedMeta::Meta(meta) => return Err(syn::Error::new(meta.span(), "offset must be an integer!")),
                        }
                    },
                    // #[offset = ...]
                    syn::Meta::NameValue(nv) => match &nv.lit {
                        syn::Lit::Int(i) => {
                            let offset = parse_literal_offset(&i.to_string())
                                .map_err(|_| syn::Error::new(i.span(), "Offset must be a valid integer!"))?;
                            hook_data = Some(HookData::from_offset(attr, offset)?);
                        },
                        _ => return Err(
                            syn::Error::new(nv.lit.span(), "Offset must be a literal integer!")
                        ),
                    },
                }
            }
        }
        Ok(Self {
            original: func,
            attrs,
            args: fn_args,
            hook_data,
        })
    }
}


impl<'a> HookData<'a> {
    fn from_offset(attribute: &'a Attribute, offset: usize) -> syn::Result<Self>{
        Ok(Self {
            original: attribute,
            offset: Some(offset),
            symbol: None,

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