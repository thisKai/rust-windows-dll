pub use windows_dll_codegen::windows_dll;

use winapi::{
    shared::{
        minwindef::{WORD, FARPROC},
    },
    um::winnt::LPCSTR,
};

#[inline]
pub unsafe fn load_dll_proc_ordinal(name: *const u16, proc_ordinal: WORD) -> Option<FARPROC> {
    use winapi::um::winuser::MAKEINTRESOURCEA;

    load_dll_proc_name(name, MAKEINTRESOURCEA(proc_ordinal))
}
#[inline]
pub unsafe fn load_dll_proc_name(name: *const u16, proc_name: LPCSTR) -> Option<FARPROC> {
    use winapi::um::libloaderapi::{LoadLibraryW, GetProcAddress};

    let library = LoadLibraryW(name);
    if library.is_null() {
        return None;
    }

    let function_pointer = GetProcAddress(library, proc_name);
    if function_pointer.is_null() {
        return None;
    }

    Some(function_pointer)
}