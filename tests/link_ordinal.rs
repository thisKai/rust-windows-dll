#[cfg(feature = "winapi")]
mod winapi {
    use windows_dll::dll;

    use winapi::shared::{minwindef::BOOL, ntdef::VOID, windef::HWND};

    #[test]
    fn link_ordinal() {
        #[dll("uxtheme.dll")]
        extern "system" {
            #[link_ordinal = 137]
            fn flush_menu_themes() -> VOID;
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
}
