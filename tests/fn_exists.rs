use windows_dll::dll;

use winapi::shared::{basetsd::SIZE_T, minwindef::BOOL, ntdef::PVOID, windef::HWND};

#[test]
fn function_exists() {
    #[dll("user32.dll")]
    extern "system" {
        #[allow(non_snake_case)]
        fn SetWindowCompositionAttribute(
            h_wnd: HWND,
            data: *mut WINDOWCOMPOSITIONATTRIBDATA,
        ) -> BOOL;
    }

    dbg!(SetWindowCompositionAttribute::exists());
}

#[test]
fn function_exists_module() {
    mod user32 {
        use super::*;
        #[dll("user32.dll")]
        extern "system" {
            #[allow(non_snake_case)]
            pub fn SetWindowCompositionAttribute(
                h_wnd: HWND,
                data: *mut WINDOWCOMPOSITIONATTRIBDATA,
            ) -> BOOL;
        }
    }
    use user32::SetWindowCompositionAttribute;

    dbg!(SetWindowCompositionAttribute::exists());
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
