use platform::*;
use windows_dll::dll;

#[test]
fn link_ordinal() {
    #[dll("uxtheme.dll")]
    extern "system" {
        #[link_ordinal = 137]
        fn flush_menu_themes();
    }
}

#[test]
fn link_ordinal_with_arguments() {
    #[dll("uxtheme.dll")]
    extern "system" {
        #[link_ordinal = 133]
        fn allow_dark_mode_for_window(hwnd: HWND, allow: BOOL) -> BOOL;
    }
}

#[cfg(feature = "winapi")]
mod platform {
    pub use winapi::shared::{minwindef::BOOL, windef::HWND};
}

#[cfg(feature = "windows-sys")]
mod platform {
    pub use windows_sys::Win32::Foundation::{BOOL, HWND};
}
