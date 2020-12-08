use std::iter::once;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    Result,
    parse,
    Ident,
    Lit,
    LitInt,
    Expr,
    ExprLit,
    ItemForeignMod,
    ForeignItem,
    ForeignItemFn,
    Signature,
    FnArg,
    Meta,
    NestedMeta,
    ReturnType,
    punctuated::Punctuated,
    token::Comma,
    parse::Parser,
    spanned::Spanned,
};
use quote::quote;
use proc_macro_crate::crate_name;

pub fn parse_windows_dll(metadata: TokenStream, input: TokenStream) -> Result<proc_macro2::TokenStream> {
    let (dll_name, load_library_ex_flags) = parse_dll_name(metadata)?;
    let functions = parse_extern_block(&dll_name, load_library_ex_flags.as_ref(), input)?;
    Ok(functions)
}

/// Extract the arguments from the #[dll] macro.
pub fn parse_dll_name(metadata: TokenStream) -> Result<(String, Option<Expr>)> {

    // Our arguments take the form of `LitStr[, Expr]?`, where the first argument
    // is the dll name, and the second arg is a flag to pass to LoadLibraryExW.
    // The easiest way to represent this is with a Punctuated list of expr,
    // which we will limit to two elements manually.
    let parser = Punctuated::<Expr, Comma>::parse_terminated;
    let args: Punctuated<Expr, Comma> = parser.parse(metadata)?;

    // Extract dll name
    let mut args_it = args.clone().into_iter();
    let dll = match args_it.next().unwrap() {
        Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value(),
        expr => return Err(syn::Error::new(expr.span(), "DLL name must be a string.")),
    };

    // Extract the library args (if they exist).
    let load_library_args = args_it.next();

    // Ensure there aren't any extra flags afterwards.
    if args_it.next().is_some() {
        return Err(syn::Error::new(args.span(), "Too many arguments passed to dll macro."));
    }

    Ok((dll, load_library_args))
}

pub fn parse_extern_block(dll_name: &str, load_library_ex_flags: Option<&Expr>, input: TokenStream) -> Result<proc_macro2::TokenStream> {
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

                // Generate the flags to pass to the load_library_ex function.
                // Defaulting to 0 will make LoadLibraryExW behave like
                // LoadLibrary, according to the docs:
                // > If no flags are specified, the behavior of this function is
                // > identical to that of the LoadLibrary function.
                // https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw
                let flags = if let Some(expr) = load_library_ex_flags {
                    quote! { #expr }
                } else {
                    quote! { 0 }
                };

                let func_ptr = match link_attr {
                    Some(Link::Ordinal(ordinal)) => quote! {
                        load_dll_proc_ordinal_ex(#dll_name, #crate_name::Proc::Ordinal(#ordinal), #wide_dll_name, #ordinal, #flags)
                    },
                    Some(Link::Name(name)) => {
                        let name_lpcstr = name.as_bytes().iter().map(|c| *c as i8).chain(once(0));
                        quote! {
                            load_dll_proc_name_ex(#dll_name, #crate_name::Proc::Name(#name), #wide_dll_name, (&[#(#name_lpcstr),*]).as_ptr(), #flags)
                        }
                    },
                    _ => {
                        let name = ident.to_string();
                        let name_lpcstr = name.as_bytes().iter().map(|c| *c as i8).chain(once(0));
                        quote! {
                            load_dll_proc_name_ex(#dll_name, #crate_name::Proc::Name(#name), #wide_dll_name, (&[#(#name_lpcstr),*]).as_ptr(), #flags)
                        }
                    },
                };

                let outer_return_type = if fallible_attr {
                    match &output {
                        ReturnType::Default => {
                            quote! { -> #crate_name::Result<(), #crate_name::Error> }
                        }
                        ReturnType::Type(_, ty) => {
                            quote! { -> #crate_name::Result<#ty, #crate_name::Error> }
                        }
                    }
                } else {
                    quote! { #output }
                };

                let get_func_ptr = if fallible_attr {
                    quote! { ::ptr_clone_err()? }
                } else {
                    quote! { ::ptr().expect(#error) }
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
                        unsafe fn ptr() -> #crate_name::Result<&'static unsafe #abi fn( #(#inputs),* ) #output, &'static #crate_name::Error> {
                            use {
                                #crate_name::{
                                    load_dll_proc_name,
                                    load_dll_proc_ordinal,
                                    once_cell::sync::OnceCell,
                                    core::mem::transmute,
                                    Result,
                                },
                            };
                            static FUNC_PTR: OnceCell<Result<unsafe #abi fn( #(#inputs),* ) #output, #crate_name::Error>> = OnceCell::new();
                            FUNC_PTR.get_or_init(|| {
                                let func_ptr = #func_ptr?;

                                Ok(transmute(func_ptr))
                            }).as_ref()
                        }
                        #[inline]
                        unsafe fn ptr_clone_err() -> #crate_name::Result<&'static unsafe #abi fn( #(#inputs),* ) #output, #crate_name::Error> {
                            Self::ptr().map_err(|err| err.clone())
                        }
                        pub fn exists() -> bool {
                            unsafe { Self::ptr().is_ok() }
                        }
                    }

                    #(#attrs)*
                    #vis unsafe fn #ident ( #(#inputs),* ) #outer_return_type {
                        let func = #ident#get_func_ptr;

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