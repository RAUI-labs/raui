extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str,
    punctuated::Punctuated,
    DeriveInput, FnArg, Ident, ItemFn, Pat, PatIdent, Path, Result, Token, Type, TypePath,
    TypeReference,
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

// The links won't be broken when built in the context of the `raui` crate
/// An attribute macro that allows you to add hooks that will execute before your component body
///
/// > **See Also:** [`macro@post_hooks`] for an alternative that runs _after_ your component body
///
/// Hooks allow you to create reusable logic that can be applied to multiple components.
///
/// # Usage Example
///
/// ```rust,ignore
/// # use raui::prelude::*;
/// #[pre_hooks(use_button_notified_state)]
/// pub fn image_button(mut context: WidgetContext) -> WidgetNode {
///    // Do stuff and potentially use state added by the `use_button_notified_state` hook
/// }
/// ```
///
/// # Creating a Hook
///
/// Hooks are simply functions that take a mutable reference to the component's
/// [`WidgetContext`][raui_core::widget::context::WidgetContext].
///
/// ```ignore
/// pub fn use_button_notified_state(context: &mut WidgetContext) {
///     // hook into the lifecycle of whatever widget this hook is applied to
///     context.life_cycle.change(|context| {
///         for msg in context.messenger.messages {
///             // listen for button messages
///             if let Some(msg) = msg.as_any().downcast_ref::<ButtonNotifyMessage>() {
///                 // And modify the context state with the button info
///                 let _ = context.state.write_with(msg.state);
///             }
///         }
///     });
/// }
/// ```
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

/// Allows you to execute re-usable logic after your component body
///
/// See [`macro@pre_hooks`]
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

// The links won't be broken when built in the context of the `raui` crate
/// Derive macro for the [`PropsData`][raui_core::props::PropsData] trait
/// 
/// # Example
///
/// ```ignore
/// #[derive(PropsData, Debug, Default, Copy, Clone, Serialize, Deserialize)]
/// #[props_data(crate::props::PropsData)]
/// #[prefab(crate::Prefab)]
/// pub struct ButtonProps {
///     #[serde(default)]
///     pub selected: bool,
///     #[serde(default)]
///     pub trigger: bool,
///     #[serde(default)]
///     pub context: bool,
///     #[serde(default)]
///     pub pointer: Vec2,
/// }
/// ```
#[proc_macro_derive(PropsData, attributes(remote, props_data, prefab))]
pub fn derive_props(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input as DeriveInput);

    let mut path = Path::from(ident);
    let mut props_data = parse_str::<Path>("PropsData").unwrap();
    let mut prefab = parse_str::<Path>("Prefab").unwrap();
    for attr in attrs {
        if let Some(ident) = attr.path.get_ident() {
            if ident == "remote" {
                path = attr.parse_args::<Path>().unwrap();
            } else if ident == "props_data" {
                props_data = attr.parse_args::<Path>().unwrap();
            } else if ident == "prefab" {
                prefab = attr.parse_args::<Path>().unwrap();
            }
        }
    }

    let tokens = quote! {
        impl #props_data for #path
        where
            Self: Clone,
        {
            fn clone_props(&self) -> Box<dyn #props_data> {
                Box::new(self.clone())
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        impl #prefab for #path {}
    };
    tokens.into()
}

// The links won't be broken when built in the context of the `raui` crate
/// Derive macro for the [`MessageData`][raui_core::messenger::MessageData] trait
/// 
/// # Example
///
/// ```ignore
/// #[derive(MessageData, Debug, Clone)]
/// pub enum AppMessage {
///     ShowPopup(usize),
///     ClosePopup,
/// }
/// ```
#[proc_macro_derive(MessageData, attributes(remote, message_data))]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input as DeriveInput);

    let mut path = Path::from(ident);
    let mut message_data = parse_str::<Path>("MessageData").unwrap();
    for attr in attrs {
        if let Some(ident) = attr.path.get_ident() {
            if ident == "remote" {
                path = attr.parse_args::<Path>().unwrap();
            } else if ident == "message_data" {
                message_data = attr.parse_args::<Path>().unwrap();
            }
        }
    }

    let tokens = quote! {
        impl #message_data for #path
        where
            Self: Clone,
        {
            fn clone_message(&self) -> Box<dyn #message_data> {
                Box::new(self.clone())
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
    tokens.into()
}
