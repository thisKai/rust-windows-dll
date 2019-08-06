use windows_dll::windows_dll;

use std::ffi::c_void;

#[test]
fn dont_panic() {
    #[windows_dll("uxtheme.dll")]
    extern "system" {
        #[link_ordinal = 135]
        fn flush_menu_themes() -> c_void;
    }
}
