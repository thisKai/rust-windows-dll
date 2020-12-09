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
/// The `.dll` file extension can be omitted if the dll name has no path, e.g
/// ```
/// #[dll("user32")]
/// extern "system" {
///     ...
/// }
/// ```
/// if the dll name is a valid rust identifier, you can also omit the quotes, e.g:
/// ```
/// #[dll(user32)]
/// extern "system" {
///     ...
/// }
/// ```
///
/// For each function declaration, an unsafe rust wrapper function will be generated
/// which dynamically loads the original function from the dll.
///
/// ## Rename
/// If you need to give the rust function a different name
/// you can manually specify the dll symbol to load,
/// Just put the dll symbol name in a **`#[link_name]`** attribute, e.g:
/// ```
/// #[dll(user32)]
/// extern "system" {
///     #[link_name = "SetWindowCompositionAttribute"]
///     fn set_window_composition_attribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
/// }
/// ```
///
/// ## Ordinal exports
/// If you need to load a function that is exported by ordinal
/// you can put the ordinal in a **`#[link_ordinal]`** attribute, e.g:
/// ```
/// #[dll(uxtheme)]
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
/// You can also generate a wrapper function which returns a `Result<T, windows_dll::Error<function_name>>`
/// To better integrate with the **`?`** operator, Just put a **`#[fallible]`** attribute
/// on the function declaration, e.g:
/// ```
/// #[dll(user32)]
/// extern "system" {
///     #[allow(non_snake_case)]
///     #[fallible]
///     fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
/// }
/// fn main() -> Result<(), Box<dyn Error>> {
///     ...
///     let result: Result<BOOL, windows_dll::Error<SetWindowCompositionAttribute>> = SetWindowCompositionAttribute(h_wnd, data);
///     ...
/// }
/// ```
///
/// # LoadLibraryExW flags
/// This library uses the Win32 API function
/// [LoadLibraryExW](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw)
/// internally. You can pass flags to the dwFlags parameter
/// by passing a second argument to the **`#[dll]`** attribute, e.g
/// ```
/// #[dll(bcrypt, LOAD_LIBRARY_SEARCH_SYSTEM32)]
/// extern "system" {
///     #[link_name = "BCryptAddContextFunction"]
///     fn bcrypt_add_context_function(dw_table: ULONG, psz_context: LPCWSTR, dw_interface: ULONG, psz_function: LPCWSTR, dw_position: ULONG) -> BOOL;
/// }
/// ```
/// Available flags are re-exported from the **`flags`** module

#[proc_macro_attribute]
pub fn dll(metadata: TokenStream, input: TokenStream) -> TokenStream {
    parse_windows_dll(metadata, input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
