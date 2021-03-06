use windows_dll::dll;

use winapi::shared::{basetsd::SIZE_T, minwindef::BOOL, ntdef::PVOID, windef::HWND};

#[test]
fn link_name() {
    #[dll("user32.dll")]
    extern "system" {
        #[link_name = "SetWindowCompositionAttribute"]
        fn set_window_composition_attribute(
            h_wnd: HWND,
            data: *mut WINDOWCOMPOSITIONATTRIBDATA,
        ) -> BOOL;
    }
}

#[allow(non_snake_case)]
type WINDOWCOMPOSITIONATTRIB = u32;

#[allow(non_snake_case)]
#[repr(C)]
pub struct WINDOWCOMPOSITIONATTRIBDATA {
    Attrib: WINDOWCOMPOSITIONATTRIB,
    pvData: PVOID,
    cbData: SIZE_T,
}
