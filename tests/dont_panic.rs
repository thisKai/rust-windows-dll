use windows_dll::windows_dll;

use winapi::shared::{
    ntdef::VOID,
    minwindef::BOOL,
    windef::HWND,
};

#[test]
fn dont_panic() {
    #[windows_dll("uxtheme.dll")]
    extern "system" {
        #[link_ordinal = 137]
        fn flush_menu_themes() -> VOID;
    }
}


#[test]
fn arguments() {
    #[windows_dll("uxtheme.dll")]
    extern "system" {
        #[link_ordinal = 133]
        fn allow_dark_mode_for_window(hwnd: HWND, allow: BOOL) -> BOOL;
    }
}
