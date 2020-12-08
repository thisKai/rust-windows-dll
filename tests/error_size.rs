use windows_dll::{dll, Error};

use winapi::shared::{
    minwindef::BOOL,
    windef::HWND,
    ntdef::PVOID,
    basetsd::SIZE_T,
};

#[test]
fn error_is_1_byte() {
    #[dll("user32.dll")]
    extern "system" {
        #[allow(non_snake_case)]
        #[fallible]
        fn SetWindowCompositionAttribute(h_wnd: HWND, data: *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
    }

    assert_eq!(core::mem::size_of::<Error<SetWindowCompositionAttribute>>(), 1);
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
