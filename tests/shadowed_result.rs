use windows_dll::dll;

use winapi::shared::{
    minwindef::BOOL,
    windef::HWND,
};

// Don't error, even if we redefine Result
type Result = core::result::Result<(), ()>;
#[dll("user32.dll")]
extern "system" {
    #[allow(non_snake_case)]
    pub fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
}
