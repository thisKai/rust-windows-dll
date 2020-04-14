mod windows_dll_impl;

extern crate proc_macro;

use windows_dll_impl::parse_windows_dll;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn windows_dll(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_windows_dll(metadata, input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
