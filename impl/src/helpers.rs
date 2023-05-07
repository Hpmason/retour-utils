
use proc_macro2::Span;
use syn::{TypeReference, Type, Token, TypePath, Path, punctuated::Punctuated, PathSegment, Ident, Attribute, token::{Bracket, PathSep}, Meta, MetaList, spanned::Spanned, AngleBracketedGenericArguments, Signature, TypeBareFn, BareFnArg, FnArg};

use crate::{fold::DetourInfo, crate_refs, parse::HookAttributeArgs};




pub fn str_type_from_span(span: Span) -> Type {
    syn::Type::Reference(TypeReference {
        and_token: Token![&](span),
        lifetime: None,
        mutability: None,
        elem: Box::new(Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: {
                    let mut p = Punctuated::new();
                    p.push(PathSegment {
                        ident: Ident::new("str", span),
                        arguments: syn::PathArguments::None,
                    });
                    p
                },
            },
        })),
    })
}

pub fn allow_attr(allows: &[String]) -> Attribute{
    Attribute {
        pound_token: Token![#](Span::call_site()),
        style: syn::AttrStyle::Outer,
        bracket_token: Bracket(Span::call_site()),
        meta: Meta::List(MetaList {
            path: Path {
                leading_colon: None,
                segments: {
                    let mut p = Punctuated::new();
                    p.push(PathSegment {
                        ident: Ident::new("allow", Span::call_site()),
                        arguments: syn::PathArguments::None,
                    });
                    p
                },
            },
            delimiter: syn::MacroDelimiter::Bracket(Bracket(Span::call_site())),
            tokens: quote::quote!{
                #(#allows),*
            },
        }),
    }
}

pub fn static_detour_ty(info: &DetourInfo) -> Type {
    let span = info.hook_attr.span();
    Type::Path(TypePath { 
        qself: None, 
        path: Path { 
            leading_colon: Some(PathSep(span)), 
            segments: {
                let mut p = Punctuated::new();
                p.extend([
                    PathSegment {
                        ident: crate_refs::retour_crate(),
                        arguments: syn::PathArguments::None,
                    },
                    PathSegment {
                        ident: Ident::new("StaticDetour", span),
                        arguments: syn::PathArguments::AngleBracketed(
                                AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Token![<](span),
                                    args: {
                                        let mut p = Punctuated::new();
                                        p.push(syn::GenericArgument::Type(fn_type(&info.fn_sig, &info.hook_attr)));
                                        p
                                    },
                                    gt_token: Token![>](span),
                                }),
                    },
                ]);
                p
            } 
        }
    })
}

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