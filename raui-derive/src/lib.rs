extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    FnArg, Ident, ItemFn, Pat, PatIdent, Result, Token, Type, TypePath, TypeReference,
};

#[derive(Debug, Clone)]
struct IdentList {
    values: Punctuated<Ident, Token![,]>,
}

impl Parse for IdentList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            values: input.parse_terminated(Ident::parse)?,
        })
    }
}

fn unpack_context(ty: &Type, pat: &Pat) -> Option<Ident> {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(segment) = path.segments.iter().last() {
                if segment.ident == "WidgetContext" {
                    if let Pat::Ident(PatIdent { ident, .. }) = pat {
                        return Some(ident.to_owned());
                    }
                }
            }
        }
        Type::Reference(TypeReference { elem, .. }) => {
            return unpack_context(&**elem, pat);
        }
        _ => {}
    }
    None
}

fn is_arg_context(arg: &FnArg) -> Option<Ident> {
    if let FnArg::Typed(pat) = arg {
        unpack_context(&*pat.ty, &*pat.pat)
    } else {
        None
    }
}

#[proc_macro_attribute]
pub fn pre_hooks(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as ItemFn);
    let context = sig
        .inputs
        .iter()
        .find_map(is_arg_context)
        .unwrap_or_else(|| panic!("Could not find function context argument!"));
    let list = parse_macro_input!(attr as IdentList);
    let hooks = list
        .values
        .into_iter()
        .map(|v| quote! { #context.use_hook(#v); });

    let tokens = quote! {
        #(#attrs)*
        #vis #sig {
            #(#hooks)*
            #block
        }
    };
    tokens.into()
}

#[proc_macro_attribute]
pub fn post_hooks(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as ItemFn);
    let context = sig
        .inputs
        .iter()
        .find_map(is_arg_context)
        .unwrap_or_else(|| panic!("Could not find function context argument!"));
    let list = parse_macro_input!(attr as IdentList);
    let hooks = list
        .values
        .into_iter()
        .map(|v| quote! { #context.use_hook(#v); });

    let tokens = quote! {
        #(#attrs)*
        #vis #sig {
            let result = {
                #block
            };
            #(#hooks)*
            result
        }
    };
    tokens.into()
}
