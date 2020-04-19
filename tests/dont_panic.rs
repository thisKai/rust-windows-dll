use windows_dll::windows_dll;

use winapi::shared::{
    ntdef::VOID,
    minwindef::BOOL,
    windef::HWND,
    ntdef::PVOID,
    basetsd::SIZE_T,
};


#[test]
fn link_ordinal() {
    #[windows_dll("uxtheme.dll")]
    extern "system" {
        #[link_ordinal = 137]
        fn flush_menu_themes() -> VOID;
    }
}


#[test]
fn link_ordinal_with_arguments() {
    #[windows_dll("uxtheme.dll")]
    extern "system" {
        #[link_ordinal = 133]
        fn allow_dark_mode_for_window(hwnd: HWND, allow: BOOL) -> BOOL;
    }
}

#[test]
fn link_name() {
    #[windows_dll("user32.dll")]
    extern "system" {
        #[link_name = "SetWindowCompositionAttribute"]
        fn set_window_composition_attribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
    }
}

#[allow(non_snake_case)]
type WINDOWCOMPOSITIONATTRIB = u32;
const WCA_USEDARKMODECOLORS: WINDOWCOMPOSITIONATTRIB = 26;

#[allow(non_snake_case)]
#[repr(C)]
struct WINDOWCOMPOSITIONATTRIBDATA {
    Attrib: WINDOWCOMPOSITIONATTRIB,
    pvData: PVOID,
    cbData: SIZE_T,
}

#[test]
fn guess_name() {
    #[windows_dll("user32.dll")]
    extern "system" {
        #[allow(non_snake_case)]
        fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
    }
}

#[test]
fn return_option() {
    #[windows_dll("user32.dll")]
    extern "system" {
        #[allow(non_snake_case)]
        #[optional]
        fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
    }
}
