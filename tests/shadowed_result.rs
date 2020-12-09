use windows_dll::dll;

use winapi::shared::{
    minwindef::BOOL,
    windef::HWND,
    ntdef::PVOID,
    basetsd::SIZE_T,
};

// Don't error, even if we redefine Result
#[allow(dead_code)]
type Result = core::result::Result<(), ()>;

#[dll("user32.dll")]
extern "system" {
    #[allow(non_snake_case)]
    pub fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
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
