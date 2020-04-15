use std::iter::once;
use proc_macro::TokenStream;
use syn::{
    Result,
    parse,
    Lit,
    LitInt,
    LitStr,
    ItemForeignMod,
    ForeignItem,
    ForeignItemFn,
    FnDecl,
    FnArg,
    Meta,
    NestedMeta,
};
use quote::quote;

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
    let ItemForeignMod { abi, items, .. } = parse(input)?;

    let functions = items.into_iter().map(|i| {
        match i {
            ForeignItem::Fn(ForeignItemFn { attrs, vis, ident, decl, .. }) => {
                let link_attr = attrs.iter().find_map(|attr| {
                    let meta = attr.parse_meta().ok()?;
                    if meta.name() == "link_ordinal" {
                        match meta_value(meta)? {
                            Lit::Int(int) => Some(Link::Ordinal(int)),
                            _ => None,
                        }
                    } else if meta.name() == "link_name" {
                        match meta_value(meta)? {
                            Lit::Str(string) => Some(Link::Name(string.value())),
                            _ => None,
                        }
                    } else {
                        None
                    }
                });

                let FnDecl { generics, inputs, variadic, output, .. } = &*decl;

                let wide_dll_name = dll_name.encode_utf16().chain(once(0));

                let argument_names = inputs.iter().map(|i| {
                    match i {
                        FnArg::Captured(arg) => &arg.pat,
                        FnArg::Inferred(pat) => pat,
                        _ => panic!("Argument type not supported"),
                    }
                });

                let error = format!("Could not load function {} from {}", &ident, dll_name);

                let wide_dll_name = quote! { (&[#(#wide_dll_name),*]).as_ptr() };

                let func_ptr = match link_attr {
                    Some(Link::Ordinal(ordinal)) => quote! {
                        load_dll_proc_ordinal(#wide_dll_name, #ordinal)
                    },
                    Some(Link::Name(name)) => {
                        let name = name.as_bytes();
                        quote! {
                            load_dll_proc_name(#wide_dll_name, (&[#(#name),*]).as_ptr() as _)
                        }
                    },
                    _ => {
                        let name = ident.to_string();
                        let name = name.as_bytes();
                        quote! {
                            load_dll_proc_name(#wide_dll_name, (&[#(#name),*]).as_ptr() as _)
                        }
                    },
                };

                quote! {
                    #vis unsafe fn #ident ( #(#inputs),* ) #output {
                        use {
                            core::mem::transmute,
                            windows_dll::{load_dll_proc_name, load_dll_proc_ordinal},
                        };

                        let func_ptr = #func_ptr.expect(#error);

                        let func: unsafe #abi fn( #(#inputs),* ) #output = transmute(func_ptr);

                        func( #(#argument_names),* )
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
                            NestedMeta::Literal(literal) => Some(literal),
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