mod windows_dll_impl;

extern crate proc_macro;

use windows_dll_impl::parse_windows_dll;
use proc_macro::TokenStream;

/// # Dynamically load functions from a windows dll
///
/// Works on extern blocks containing only functions, e.g:
/// ```
/// #[dll("user32.dll")]
/// extern "system" {
///     #[allow(non_snake_case)]
///     fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
/// }
/// ```
/// For each function declaration, an unsafe rust wrapper function will be generated
/// which dynamically loads the original function from the dll.
///
/// ## Rename
/// If you need to give the rust function a different name
/// you can manually specify the dll symbol to load,
/// Just put the dll symbol name in a **#\[link_name\]** attribute, e.g:
/// ```
/// #[dll("user32.dll")]
/// extern "system" {
///     #[link_name = "SetWindowCompositionAttribute"]
///     fn set_window_composition_attribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
/// }
/// ```
///
/// ## Ordinal exports
/// If you need to load a function that is exported by ordinal
/// you can put the ordinal in a **#\[link_ordinal\]** attribute, e.g:
/// ```
/// #[dll("uxtheme.dll")]
/// extern "system" {
///     #[link_ordinal = 133]
///     fn allow_dark_mode_for_window(hwnd: HWND, allow: BOOL) -> BOOL;
/// }
/// ```
///
/// ## Error handling
/// By default the generated functions panic when the dll function cannot be loaded
/// you can check if they exist by calling `function_name::exists()` which returns a `bool`, e.g:
/// ```
/// if allow_dark_mode_for_window::exists() {
///     allow_dark_mode_for_window(hwnd, allow)
/// }
/// ```
///
/// You can also generate a wrapper function which returns a Result<T, windows_dll::Error>
/// To better integrate with the **?** operator, Just put a **#\[fallible\]** attribute
/// on the function declaration, e.g:
/// ```
/// #[dll("user32.dll")]
/// extern "system" {
///     #[allow(non_snake_case)]
///     #[fallible]
///     fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
/// }
/// fn main() -> Result<(), Box<dyn Error>> {
///     ...
///     SetWindowCompositionAttribute(h_wnd, data)?;
///     ...
/// }
/// ```
///

#[proc_macro_attribute]
pub fn dll(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_windows_dll(metadata, input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
