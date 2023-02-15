mod expand;
mod types;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemMod};

#[proc_macro_attribute]
pub fn hook_module(args: TokenStream, input: TokenStream) -> TokenStream {

    let ast = parse_macro_input!(input as ItemMod);
    let args = parse_macro_input!(args as AttributeArgs);
    
    if args.len() != 1 {
        return syn::Error::new(
                ast.mod_token.span, 
                "#[hook_module] expects the name of the library module to hook into. Try: #[hook_module(\"<dll_name>\")]"
            )
            .into_compile_error()
            .into()
    }
    // eprintln!("{ast:#?}");
    // eprintln!("{args:#?}");
    
    let stream = expand::expand(&ast, &args.first().expect("Already checked size"))
        .unwrap_or_else(syn::Error::into_compile_error);
    stream.into()
}