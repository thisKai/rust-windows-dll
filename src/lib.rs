pub use windows_dll_codegen::windows_dll;

use winapi::shared::{
    minwindef::{WORD, FARPROC},
};

#[inline]
pub unsafe fn load_dll_proc(name: *const u16, link_ordinal: WORD) -> FARPROC {
    use winapi::um::{
        libloaderapi::{LoadLibraryW, GetProcAddress},
        winuser::MAKEINTRESOURCEA,
    };

    let library = LoadLibraryW(name);

    GetProcAddress(library, MAKEINTRESOURCEA(link_ordinal))
}