use platform::*;
use windows_dll::{dll, Error};

#[test]
fn error_is_1_byte() {
    #[dll("user32.dll")]
    extern "system" {
        #[allow(non_snake_case)]
        #[fallible]
        fn SetWindowCompositionAttribute(
            h_wnd: HWND,
            data: *mut WINDOWCOMPOSITIONATTRIBDATA,
        ) -> BOOL;
    }

    assert_eq!(
        core::mem::size_of::<Error<SetWindowCompositionAttribute>>(),
        1
    );
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

#[cfg(feature = "winapi")]
mod platform {
    pub use winapi::shared::{basetsd::SIZE_T, minwindef::BOOL, ntdef::PVOID, windef::HWND};
}

#[cfg(feature = "windows")]
mod platform {
    use core::ffi::c_void;
    pub use windows::Win32::Foundation::{BOOL, HWND};

    pub type PVOID = *mut c_void;
    #[allow(non_camel_case_types)]
    pub type SIZE_T = usize;
}
