use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::crate_name;
use quote::quote;
use std::iter::once;
use syn::{
    parse, parse::Parser, punctuated::Punctuated, spanned::Spanned, token::Comma, Expr, ExprLit,
    ExprPath, FnArg, ForeignItem, ForeignItemFn, Ident, ItemForeignMod, Lit, LitInt, Meta,
    NestedMeta, Result, ReturnType, Signature,
};

pub fn parse_windows_dll(
    metadata: TokenStream,
    input: TokenStream,
) -> Result<proc_macro2::TokenStream> {
    let (dll_name, load_library_ex_flags) = parse_attribute_args(metadata)?;
    let functions = parse_extern_block(&dll_name, load_library_ex_flags.as_ref(), input)?;
    Ok(functions)
}

/// Extract the arguments from the #[dll] macro.
pub fn parse_attribute_args(metadata: TokenStream) -> Result<(String, Option<Expr>)> {
    // Our arguments take the form of `LitStr[, Expr]?`, where the first argument
    // is the dll name, and the second arg is a flag to pass to LoadLibraryExW.
    // The easiest way to represent this is with a Punctuated list of expr,
    // which we will limit to two elements manually.
    let parser = Punctuated::<Expr, Comma>::parse_terminated;
    let args: Punctuated<Expr, Comma> = parser.parse(metadata)?;

    // Extract dll name
    let error_text = "DLL name must be a string or identifier";
    let mut args_it = args.clone().into_iter();
    let dll = match args_it.next().unwrap() {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => s.value(),
        Expr::Path(ExprPath { path, .. }) => match path.get_ident() {
            Some(ident) => ident.to_string(),
            None => return Err(syn::Error::new(path.span(), error_text)),
        },
        expr => return Err(syn::Error::new(expr.span(), error_text)),
    };

    // Extract the library args (if they exist).
    let load_library_args = args_it.next();

    // Ensure there aren't any extra flags afterwards.
    if args_it.next().is_some() {
        return Err(syn::Error::new(
            args.span(),
            "Too many arguments passed to dll macro.",
        ));
    }

    Ok((dll, load_library_args))
}

pub fn parse_extern_block(
    dll_name: &str,
    load_library_ex_flags: Option<&Expr>,
    input: TokenStream,
) -> Result<proc_macro2::TokenStream> {
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

                let wide_dll_name = quote! { (&[#(#wide_dll_name),*]).as_ptr() };

                let link = link_attr.unwrap_or_else(|| Link::Name(ident.to_string()));
                let fn_ptr = quote! { #crate_name::load_dll_proc::<#ident>() };

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

                let outer_return_type = if fallible_attr {
                    match &output {
                        ReturnType::Default => {
                            quote! { -> #crate_name::Result<(), #crate_name::Error<#ident>> }
                        }
                        ReturnType::Type(_, ty) => {
                            quote! { -> #crate_name::Result<#ty, #crate_name::Error<#ident>> }
                        }
                    }
                } else {
                    quote! { #output }
                };

                let get_fn_ptr = if fallible_attr {
                    quote! {
                        match #ident::ptr() {
                            Ok(fn_ptr) => fn_ptr,
                            Err(err) => return Err(*err),
                        }
                    }
                } else {
                    quote! {
                        match #ident::ptr() {
                            Ok(fn_ptr) => fn_ptr,
                            Err(err) => panic!("{}", err),
                        }
                    }
                };

                let return_value = quote! { func( #(#argument_names),* ) };
                let return_value = if fallible_attr {
                    quote! { Ok(#return_value) }
                } else {
                    return_value
                };
                let proc = link.proc(&crate_name);
                let proc_lpcstr = link.proc_lpcstr(&crate_name);

                quote! {
                    #[allow(non_camel_case_types)]
                    #vis enum #ident {}
                    impl #ident {
                        #[inline]
                        unsafe fn ptr() -> &'static #crate_name::Result<unsafe #abi fn( #(#inputs),* ) #output, #crate_name::Error<#ident>> {
                            use {
                                #crate_name::{
                                    once_cell::sync::OnceCell,
                                    core::mem::transmute,
                                    Result,
                                },
                            };
                            static FUNC_PTR: OnceCell<#crate_name::Result<unsafe #abi fn( #(#inputs),* ) #output, #crate_name::Error<#ident>>> = OnceCell::new();
                            FUNC_PTR.get_or_init(|| {
                                let func_ptr = #fn_ptr?;

                                Ok(transmute(func_ptr))
                            })
                        }
                        pub fn exists() -> bool {
                            unsafe { Self::ptr().is_ok() }
                        }
                    }

                    impl #crate_name::DllProc for #ident {
                        const LIB: &'static str = #dll_name;
                        const LIB_LPCWSTR: #crate_name::LPCWSTR = #wide_dll_name;
                        const PROC: #crate_name::Proc = #proc;
                        const PROC_LPCSTR: #crate_name::LPCSTR = #proc_lpcstr;
                        const FLAGS: #crate_name::DWORD = #flags;
                    }

                    #(#attrs)*
                    #vis unsafe fn #ident ( #(#inputs),* ) #outer_return_type {
                        let func = #get_fn_ptr;

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
impl Link {
    fn proc(&self, crate_name: &Ident) -> proc_macro2::TokenStream {
        match self {
            Self::Ordinal(ordinal) => quote! { #crate_name::Proc::Ordinal(#ordinal) },
            Self::Name(name) => quote! { #crate_name::Proc::Name(#name) },
        }
    }
    fn proc_lpcstr(&self, crate_name: &Ident) -> proc_macro2::TokenStream {
        match self {
            Self::Ordinal(ordinal) => quote! { #crate_name::make_int_resource_a(#ordinal) },
            Self::Name(name) => {
                let name_lpcstr = name.as_bytes().iter().map(|c| *c as i8).chain(once(0));
                quote! { (&[#(#name_lpcstr),*]).as_ptr() }
            }
        }
    }
}

fn meta_value(meta: Meta) -> Option<Lit> {
    match meta {
        Meta::List(mut list) => {
            if list.nested.len() == 1 {
                list.nested.pop().and_then(|pair| match pair.into_value() {
                    NestedMeta::Lit(literal) => Some(literal),
                    _ => None,
                })
            } else {
                None
            }
        }
        Meta::NameValue(name_value) => Some(name_value.lit),
        _ => None,
    }
}
