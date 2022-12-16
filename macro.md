# Dynamically load functions from a windows dll

Works on extern blocks containing only functions:

```rust
# use platform::*;
use windows_dll::dll;

#[dll("user32.dll")]
extern "system" {
    #[allow(non_snake_case)]
    fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
}
#
# #[allow(non_snake_case)]
# type WINDOWCOMPOSITIONATTRIB = u32;
#
# #[allow(non_snake_case)]
# #[repr(C)]
# pub struct WINDOWCOMPOSITIONATTRIBDATA {
#     Attrib: WINDOWCOMPOSITIONATTRIB,
#     pvData: PVOID,
#     cbData: SIZE_T,
# }
#
# #[cfg(feature = "winapi")]
# mod platform {
#     pub use winapi::shared::{basetsd::SIZE_T, minwindef::BOOL, ntdef::PVOID, windef::HWND};
# }
#
# #[cfg(feature = "windows")]
# mod platform {
#     use core::ffi::c_void;
#     pub use windows::Win32::Foundation::{BOOL, HWND};
#
#     pub type PVOID = *mut c_void;
#     #[allow(non_camel_case_types)]
#     pub type SIZE_T = usize;
# }
#
# #[cfg(feature = "windows-sys")]
# mod platform {
#     use core::ffi::c_void;
#     pub use windows_sys::Win32::Foundation::{BOOL, HWND};
#
#     pub type PVOID = *mut c_void;
#     #[allow(non_camel_case_types)]
#     pub type SIZE_T = usize;
# }
```

The `.dll` file extension can be omitted if the dll name has no path:

```rust
use windows_dll::dll;

#[dll("user32")]
extern "system" {
    // ...
}
```

if the dll name is a valid rust identifier, you can also omit the quotes:

```rust
use windows_dll::dll;

#[dll(user32)]
extern "system" {
    // ...
}
```

For each function declaration, an unsafe rust wrapper function will be generated
which dynamically loads the original function from the dll.

## Rename

If you need to give the rust function a different name
you can manually specify the dll symbol to load,
Just put the dll symbol name in a **`#[link_name]`** attribute:

```rust
# use platform::*;
use windows_dll::dll;

#[dll(user32)]
extern "system" {
    #[link_name = "SetWindowCompositionAttribute"]
    fn set_window_composition_attribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
}
#
# #[allow(non_snake_case)]
# type WINDOWCOMPOSITIONATTRIB = u32;
#
# #[allow(non_snake_case)]
# #[repr(C)]
# pub struct WINDOWCOMPOSITIONATTRIBDATA {
#     Attrib: WINDOWCOMPOSITIONATTRIB,
#     pvData: PVOID,
#     cbData: SIZE_T,
# }
#
# #[cfg(feature = "winapi")]
# mod platform {
#     pub use winapi::shared::{basetsd::SIZE_T, minwindef::BOOL, ntdef::PVOID, windef::HWND};
# }
#
# #[cfg(feature = "windows")]
# mod platform {
#     use core::ffi::c_void;
#     pub use windows::Win32::Foundation::{BOOL, HWND};
#
#     pub type PVOID = *mut c_void;
#     #[allow(non_camel_case_types)]
#     pub type SIZE_T = usize;
# }
#
# #[cfg(feature = "windows-sys")]
# mod platform {
#     use core::ffi::c_void;
#     pub use windows_sys::Win32::Foundation::{BOOL, HWND};
#
#     pub type PVOID = *mut c_void;
#     #[allow(non_camel_case_types)]
#     pub type SIZE_T = usize;
# }
```

## Ordinal exports

If you need to load a function that is exported by ordinal
you can put the ordinal in a **`#[link_ordinal]`** attribute:

```rust
# use platform::*;
use windows_dll::dll;

#[dll(uxtheme)]
extern "system" {
    #[link_ordinal = 133]
    fn allow_dark_mode_for_window(hwnd: HWND, allow: BOOL) -> BOOL;
}
#
# #[cfg(feature = "winapi")]
# mod platform {
#     pub use winapi::shared::{minwindef::BOOL, windef::HWND};
# }
#
# #[cfg(feature = "windows")]
# mod platform {
#     pub use windows::Win32::Foundation::{BOOL, HWND};
# }
#
# #[cfg(feature = "windows-sys")]
# mod platform {
#     pub use windows_sys::Win32::Foundation::{BOOL, HWND};
# }
```

## Error handling

By default the generated functions panic when the dll function cannot be loaded
you can check if they exist by calling `function_name::exists()` which returns a `bool`:

```rust,no_run
# use platform::*;
# use windows_dll::dll;
#
# #[dll(uxtheme)]
# extern "system" {
#     #[link_ordinal = 133]
#     fn allow_dark_mode_for_window(hwnd: HWND, allow: BOOL) -> BOOL;
# }
#
# #[cfg(feature = "winapi")]
# mod platform {
#     pub use winapi::shared::{minwindef::{BOOL, TRUE}, windef::HWND};
#     pub const EXAMPLE_HWND: HWND = core::ptr::null_mut();
# }
#
# #[cfg(feature = "windows")]
# mod platform {
#     pub use windows::Win32::Foundation::{BOOL, HWND};
#     pub const EXAMPLE_HWND: HWND = HWND(0);
#     pub const TRUE: BOOL = BOOL(1);
# }
#
# #[cfg(feature = "windows-sys")]
# mod platform {
#     pub use windows_sys::Win32::Foundation::{BOOL, HWND};
#     pub const EXAMPLE_HWND: HWND = 0;
#     pub const TRUE: BOOL = 1;
# }
#
# let hwnd = EXAMPLE_HWND;
# let allow = TRUE;
unsafe {
    if allow_dark_mode_for_window::exists() {
        allow_dark_mode_for_window(hwnd, allow);
    }
}
```

You can also generate a wrapper function which returns a `Result<T, windows_dll::Error<function_name>>`
To better integrate with the **`?`** operator, Just put a **`#[fallible]`** attribute
on the function declaration:

```rust,no_run
# use platform::*;
use std::error::Error;
use windows_dll::dll;

#[dll(user32)]
extern "system" {
    #[allow(non_snake_case)]
    #[fallible]
    fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
}
fn main() -> Result<(), Box<dyn Error>> {
#     let hwnd = EXAMPLE_HWND;
#     let mut is_dark_mode_bigbool = TRUE;
#     let mut data = WINDOWCOMPOSITIONATTRIBDATA {
#         Attrib: WCA_USEDARKMODECOLORS,
#         pvData: &mut is_dark_mode_bigbool as *mut _ as _,
#         cbData: core::mem::size_of::<BOOL>(),
#     };
    // ...
    let result: Result<BOOL, windows_dll::Error<SetWindowCompositionAttribute>> = unsafe { SetWindowCompositionAttribute(hwnd, &mut data) };
    let result = result?;
    // ...
#     Ok(())
}
#
# #[allow(non_snake_case)]
# type WINDOWCOMPOSITIONATTRIB = u32;
# const WCA_USEDARKMODECOLORS: WINDOWCOMPOSITIONATTRIB = 26;
#
# #[allow(non_snake_case)]
# #[repr(C)]
# pub struct WINDOWCOMPOSITIONATTRIBDATA {
#     Attrib: WINDOWCOMPOSITIONATTRIB,
#     pvData: PVOID,
#     cbData: SIZE_T,
# }
#
# #[cfg(feature = "winapi")]
# mod platform {
#     pub use winapi::shared::{basetsd::SIZE_T, minwindef::{BOOL, TRUE}, ntdef::PVOID, windef::HWND};
#
#     pub const EXAMPLE_HWND: HWND = core::ptr::null_mut();
# }
#
# #[cfg(feature = "windows")]
# mod platform {
#     use core::ffi::c_void;
#     pub use windows::Win32::Foundation::{BOOL, HWND};
#
#     pub type PVOID = *mut c_void;
#     #[allow(non_camel_case_types)]
#     pub type SIZE_T = usize;
#
#     pub const EXAMPLE_HWND: HWND = HWND(0);
#     pub const TRUE: BOOL = BOOL(1);
# }
#
# #[cfg(feature = "windows-sys")]
# mod platform {
#     use core::ffi::c_void;
#     pub use windows_sys::Win32::Foundation::{BOOL, HWND};
#
#     pub type PVOID = *mut c_void;
#     #[allow(non_camel_case_types)]
#     pub type SIZE_T = usize;
#
#     pub const EXAMPLE_HWND: HWND = 0;
#     pub const TRUE: BOOL = 1;
# }
```

# LoadLibraryExW flags

This library uses the Win32 API function
[LoadLibraryExW](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw)
internally. You can pass flags to the dwFlags parameter
by passing a second argument to the **`#[dll]`** attribute:

```rust
# use platform::*;
use windows_dll::{dll, flags::*};

#[dll(bcrypt, LOAD_LIBRARY_SEARCH_SYSTEM32)]
extern "system" {
    #[link_name = "BCryptAddContextFunction"]
    fn bcrypt_add_context_function(dw_table: ULONG, psz_context: LPCWSTR, dw_interface: ULONG, psz_function: LPCWSTR, dw_position: ULONG) -> BOOL;
}
#
# #[cfg(feature = "winapi")]
# mod platform {
#     pub use winapi::shared::{
#         minwindef::{BOOL, ULONG},
#         ntdef::LPCWSTR,
#     };
# }
#
# #[cfg(feature = "windows")]
# mod platform {
#     pub use windows::{core::PCWSTR as LPCWSTR, Win32::Foundation::BOOL};
#
#     pub type ULONG = u32;
# }
#
# #[cfg(feature = "windows-sys")]
# mod platform {
#     pub use windows_sys::{core::PCWSTR as LPCWSTR, Win32::Foundation::BOOL};
#
#     pub type ULONG = u32;
# }
```

Available flags are re-exported from the **`flags`** module
