
use syn::{Type, Signature, TypeBareFn, BareFnArg, FnArg};

use crate::parse::HookAttributeArgs;

pub fn fn_type(fn_sig: &Signature, hook_info: &HookAttributeArgs) -> Type {
    Type::BareFn(TypeBareFn {
        lifetimes: None,
        unsafety: hook_info.unsafety,
        abi: hook_info.abi.clone(),
        fn_token: fn_sig.fn_token,
        paren_token: fn_sig.paren_token,
        inputs: fn_sig.inputs.clone().into_iter().map(|arg| {
            match arg {
                
                FnArg::Typed(arg) => BareFnArg {
                    attrs: arg.attrs,
                    name: None,
                    ty: *arg.ty,
                },
                FnArg::Receiver(_) => todo!(),
            }
            
        }).collect(),
        variadic: fn_sig.variadic.clone().map(|var| {
            syn::BareVariadic { 
                attrs: var.attrs, 
                name: None, 
                dots: var.dots, 
                comma: var.comma 
            }
        }),
        output: fn_sig.output.clone(),
    })
}