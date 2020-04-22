use std::iter::once;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    Result,
    parse,
    Ident,
    Lit,
    LitInt,
    LitStr,
    ItemForeignMod,
    ForeignItem,
    ForeignItemFn,
    Signature,
    FnArg,
    Meta,
    NestedMeta,
    ReturnType,
};
use quote::quote;
use proc_macro_crate::crate_name;

pub fn parse_windows_dll(metadata: TokenStream, input: TokenStream) -> Result<proc_macro2::TokenStream> {
    let dll_name = parse_dll_name(metadata)?;
    let functions = parse_extern_block(&dll_name, input)?;
    Ok(functions)
}

pub fn parse_dll_name(metadata: TokenStream) -> Result<String> {
    let dll_name: LitStr = parse(metadata)?;
    Ok(dll_name.value())
}

pub fn parse_extern_block(dll_name: &str, input: TokenStream) -> Result<proc_macro2::TokenStream> {
    let crate_name = crate_name("windows-dll").unwrap_or_else(|_| "windows_dll".to_string());
    let crate_name = Ident::new(&crate_name, Span::call_site());

    let ItemForeignMod { abi, items, .. } = parse(input)?;

    let functions = items.into_iter().map(|i| {
        match i {
            ForeignItem::Fn(ForeignItemFn { attrs, vis, sig, .. }) => {
                let link_attr = attrs.iter().find_map(|attr| {
                    let meta = attr.parse_meta().ok()?;
                    if meta.path().is_ident("link_ordinal") {
                        match meta_value(meta)? {
                            Lit::Int(int) => Some(Link::Ordinal(int)),
                            _ => None,
                        }
                    } else if meta.path().is_ident("link_name") {
                        match meta_value(meta)? {
                            Lit::Str(string) => Some(Link::Name(string.value())),
                            _ => None,
                        }
                    } else {
                        None
                    }
                });

                let fallible_attr = attrs.iter().find(|attr| {
                    match attr.parse_meta() {
                        Ok(meta) => meta.path().is_ident("fallible"),
                        Err(_) => false,
                    }
                }).is_some();

                let attrs = attrs.into_iter().filter_map(|attr| {
                        match attr.parse_meta() {
                            Ok(meta) => {
                                let path = meta.path();
                                if path.is_ident("link_ordinal") || path.is_ident("link_name") || path.is_ident("fallible") {
                                    None
                                } else {
                                    Some(attr)
                                }
                            }
                            Err(_) => {
                                Some(attr)
                            }
                        }
                    });

                let Signature { ident, inputs, output, .. } = &sig;

                let wide_dll_name = dll_name.encode_utf16().chain(once(0));
                use syn::{Pat, PatType, PatIdent};
                let argument_names = inputs.iter().map(|i| {
                    match i {
                        FnArg::Typed(PatType { pat, .. }) => match &**pat {
                            Pat::Ident(PatIdent { ident, .. }) => ident,
                            _ => panic!("Argument type not supported"),
                        }
                        _ => panic!("Argument type not supported"),
                    }
                });
                let inputs: Vec<_> = inputs.into_iter().collect();

                let error = format!("Could not load function {} from {}", &ident, dll_name);

                let wide_dll_name = quote! { (&[#(#wide_dll_name),*]).as_ptr() };

                let func_ptr = match link_attr {
                    Some(Link::Ordinal(ordinal)) => quote! {
                        load_dll_proc_ordinal(#dll_name, #crate_name::Proc::Ordinal(#ordinal), #wide_dll_name, #ordinal)
                    },
                    Some(Link::Name(name)) => {
                        let name_lpcstr = name.as_bytes().iter().map(|c| *c as i8).chain(once(0));
                        quote! {
                            load_dll_proc_name(#dll_name, #crate_name::Proc::Name(#name), #wide_dll_name, (&[#(#name_lpcstr),*]).as_ptr())
                        }
                    },
                    _ => {
                        let name = ident.to_string();
                        let name_lpcstr = name.as_bytes().iter().map(|c| *c as i8).chain(once(0));
                        quote! {
                            load_dll_proc_name(#dll_name, #crate_name::Proc::Name(#name), #wide_dll_name, (&[#(#name_lpcstr),*]).as_ptr())
                        }
                    },
                };

                let outer_return_type = if fallible_attr {
                    match &output {
                        ReturnType::Default => {
                            quote! { -> Result<(), #crate_name::Error> }
                        }
                        ReturnType::Type(_, ty) => {
                            quote! { -> Result<#ty, #crate_name::Error> }
                        }
                    }
                } else {
                    quote! { #output }
                };

                let handle_import_error = if fallible_attr {
                    quote! { ? }
                } else {
                    quote! { .expect(#error) }
                };

                let return_value = quote! { func( #(#argument_names),* ) };
                let return_value = if fallible_attr {
                    quote! { Ok(#return_value) }
                } else {
                    return_value
                };

                quote! {
                    #[allow(non_camel_case_types)]
                    #vis enum #ident {}
                    impl #ident {
                        #[inline]
                        unsafe fn ptr() -> Result<&'static unsafe #abi fn( #(#inputs),* ) #output, #crate_name::Error> {
                            use {
                                core::mem::transmute,
                                windows_dll::{
                                    load_dll_proc_name,
                                    load_dll_proc_ordinal,
                                    once_cell::sync::OnceCell,
                                },
                            };
                            static FUNC_PTR: OnceCell<Result<unsafe #abi fn( #(#inputs),* ) #output, #crate_name::Error>> = OnceCell::new();
                            FUNC_PTR.get_or_init(|| {
                                let func_ptr = #func_ptr?;

                                Ok(transmute(func_ptr))
                            }).as_ref().map_err(|err| err.clone())
                        }
                    }

                    #(#attrs)*
                    #vis unsafe fn #ident ( #(#inputs),* ) #outer_return_type {
                        let func = #ident::ptr()#handle_import_error;

                        #return_value
                    }
                }
            },
            _ => panic!("Not a function"),
        }
    });
    Ok(functions.collect())
}

enum Link {
    Ordinal(LitInt),
    Name(String),
}

fn meta_value(meta: Meta) -> Option<Lit> {
    match meta {
        Meta::List(mut list) => {
            if list.nested.len() == 1 {
                list
                    .nested
                    .pop()
                    .and_then(|pair| {
                        match pair.into_value() {
                            NestedMeta::Lit(literal) => Some(literal),
                            _ => None,
                        }
                    })
            } else {
                None
            }
        },
        Meta::NameValue(name_value) => {
            Some(name_value.lit)
        },
        _ => None,
    }
}