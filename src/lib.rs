extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn windows_dll(metadata: TokenStream, input: TokenStream) -> TokenStream {
    unimplemented!("{:?}", metadata)
}
