mod windows_dll_impl;

extern crate proc_macro;

use proc_macro::TokenStream;
use windows_dll_impl::parse_windows_dll;

#[proc_macro_attribute]
pub fn dll(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_windows_dll(metadata, input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
