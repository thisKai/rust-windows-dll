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
    let (dll_name, dll_name_span, load_library_ex_flags) = parse_attribute_args(metadata)?;
    let functions = parse_extern_block(
        &dll_name,
        dll_name_span,
        load_library_ex_flags.as_ref(),
        input,
    )?;
    Ok(functions)
}

/// Extract the arguments from the #[dll] macro.
pub fn parse_attribute_args(metadata: TokenStream) -> Result<(String, Span, Option<Expr>)> {
    // Our arguments take the form of `LitStr[, Expr]?`, where the first argument
    // is the dll name, and the second arg is a flag to pass to LoadLibraryExW.
    // The easiest way to represent this is with a Punctuated list of expr,
    // which we will limit to two elements manually.
    let parser = Punctuated::<Expr, Comma>::parse_terminated;
    let args: Punctuated<Expr, Comma> = parser.parse(metadata)?;

    // Extract dll name
    let error_text = "DLL name must be a string or identifier";
    let mut args_it = args.clone().into_iter();
    let (dll, dll_span) = match args_it.next().unwrap() {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => (s.value(), s.span()),
        Expr::Path(ExprPath { path, .. }) => match path.get_ident() {
            Some(ident) => (ident.to_string(), ident.span()),
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

    Ok((dll, dll_span, load_library_args))
}

pub fn parse_extern_block(
    dll_name: &str,
    dll_name_span: Span,
    load_library_ex_flags: Option<&Expr>,
    input: TokenStream,
) -> Result<proc_macro2::TokenStream> {
    let wide_dll_name = dll_name.encode_utf16().chain(once(0));
    let wide_dll_name = quote! { (&[#(#wide_dll_name),*]).as_ptr() };

    let crate_name = crate_name("windows-dll").unwrap_or_else(|_| "windows_dll".to_string());
    let crate_name = Ident::new(&crate_name, Span::call_site());

    let dll_type_name = if dll_name.ends_with(".dll") {
        let mut pieces = dll_name.rsplitn(3, |c| c == '.' || c == '\\' || c == '/');
        let _ext = pieces.next().unwrap();
        pieces.next().unwrap()
    } else {
        let mut pieces = dll_name.rsplitn(3, |c| c == '\\' || c == '/');

        pieces.next().unwrap()
    };
    let dll_type_ident = Ident::new(dll_type_name, dll_name_span);

    // Generate the flags to pass to the load_library_ex function.
    // Defaulting to 0 will make LoadLibraryExW behave like
    // LoadLibrary, according to the docs:
    // > If no flags are specified, the behavior of this function is
    // > identical to that of the LoadLibrary function.
    // https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw
    let flags = if let Some(expr) = load_library_ex_flags {
        quote! { #expr }
    } else {
        quote! { #crate_name::flags::NO_FLAGS }
    };

    let ItemForeignMod { abi, items, .. } = parse(input)?;

    let len = items.len();
    let dll_impl = quote! {
        #[allow(non_camel_case_types)]
        pub enum #dll_type_ident {}
        impl #crate_name::WindowsDll for #dll_type_ident {
            const LEN: usize = #len;
            const LIB: &'static str = #dll_name;
            const LIB_LPCWSTR: #crate_name::LPCWSTR = #wide_dll_name;
            const FLAGS: #crate_name::flags::LOAD_LIBRARY_FLAGS = #flags;

            unsafe fn cache() -> &'static #crate_name::DllCache<Self> {
                static LIB_CACHE: #crate_name::DllCache<#dll_type_ident> = #crate_name::DllCache::empty();

                &LIB_CACHE
            }
        }
    };

    let functions = items.into_iter().enumerate().map(|(index, item)| match item {
        ForeignItem::Fn(ForeignItemFn {
            attrs, vis, sig, ..
        }) => {
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

            let fallible_attr = attrs.iter().any(|attr| match attr.parse_meta() {
                Ok(meta) => meta.path().is_ident("fallible"),
                Err(_) => false,
            });

            let attrs = attrs.into_iter().filter(|attr| match attr.parse_meta() {
                Ok(meta) => {
                    let path = meta.path();
                    !(path.is_ident("link_ordinal")
                        || path.is_ident("link_name")
                        || path.is_ident("fallible"))
                }
                Err(_) => true,
            });

            let Signature {
                ident,
                inputs,
                output,
                ..
            } = &sig;

            use syn::{Pat, PatIdent, PatType};
            let argument_names = inputs.iter().map(|i| match i {
                FnArg::Typed(PatType { pat, .. }) => match &**pat {
                    Pat::Ident(PatIdent { ident, .. }) => ident,
                    _ => panic!("Argument type not supported"),
                },
                _ => panic!("Argument type not supported"),
            });
            let inputs: Vec<_> = inputs.into_iter().collect();

            let link = link_attr.unwrap_or_else(|| Link::Name(ident.to_string()));

            let outer_return_type = if fallible_attr {
                match &output {
                    ReturnType::Default => {
                        quote! { -> #crate_name::macro_internal::Result<(), #crate_name::Error<#ident>> }
                    }
                    ReturnType::Type(_, ty) => {
                        quote! { -> #crate_name::macro_internal::Result<#ty, #crate_name::Error<#ident>> }
                    }
                }
            } else {
                quote! { #output }
            };

            let get_fn_ptr = if fallible_attr {
                quote! {
                    <#ident as #crate_name::WindowsDllProc>::proc()?
                }
            } else {
                quote! {
                    <#ident as #crate_name::WindowsDllProc>::proc()
                        .unwrap_or_else(|err| panic!("{}", err))
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
                    pub unsafe fn exists() -> bool {
                        <Self as #crate_name::WindowsDllProc>::exists()
                    }
                }

                impl #crate_name::WindowsDllProc for #ident {
                    type Dll = #dll_type_ident;
                    type Sig = unsafe #abi fn( #(#inputs),* ) #output;
                    const CACHE_INDEX: usize = #index;
                    const PROC: #crate_name::Proc = #proc;
                    const PROC_LPCSTR: #crate_name::LPCSTR = #proc_lpcstr;

                    unsafe fn proc() -> #crate_name::macro_internal::Result<Self::Sig, #crate_name::Error<#ident>> {
                        <Self::Dll as #crate_name::WindowsDll>::cache().get_proc::<#ident>()
                    }
                }

                #(#attrs)*
                #vis unsafe fn #ident ( #(#inputs),* ) #outer_return_type {
                    let func = #get_fn_ptr;

                    #return_value
                }
            }
        }
        _ => panic!("Not a function"),
    });

    Ok(quote! {
        #dll_impl
        #(#functions)*
    })
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
            Self::Ordinal(ordinal) => {
                quote! { #crate_name::macro_internal::make_int_resource_a(#ordinal) }
            }
            Self::Name(name) => {
                let name_lpcstr = name.bytes().chain(once(0));
                quote! { (&[#(#name_lpcstr),*]).as_ptr() as _ }
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
