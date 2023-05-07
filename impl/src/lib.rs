mod expand;
mod parse;
mod crate_refs;
mod fold;
mod helpers;

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr, ItemMod};

#[proc_macro_attribute]
pub fn hook_module(args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as ItemMod);
    let args = parse_macro_input!(args as LitStr);
    
    let stream = expand::expand(ast, args)
        .unwrap_or_else(syn::Error::into_compile_error);
    stream.into()
}