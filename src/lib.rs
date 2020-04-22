pub use {
    windows_dll_codegen::dll,
    once_cell,
};

use winapi::{
    shared::{
        minwindef::{WORD, FARPROC},
    },
    um::winnt::{LPCSTR, LPCWSTR},
};

#[inline]
pub unsafe fn load_dll_proc_ordinal(name: &'static str, proc: Proc, name_lpcwstr: LPCWSTR, proc_ordinal: WORD) -> Result<FARPROC, Error> {
    use winapi::um::winuser::MAKEINTRESOURCEA;

    load_dll_proc_name(name, proc, name_lpcwstr, MAKEINTRESOURCEA(proc_ordinal))
}
#[inline]
pub unsafe fn load_dll_proc_name(name: &'static str, proc: Proc, name_lpcwstr: LPCWSTR, proc_name: LPCSTR) -> Result<FARPROC, Error> {
    use winapi::um::libloaderapi::{LoadLibraryW, GetProcAddress};

    let library = LoadLibraryW(name_lpcwstr);
    if library.is_null() {
        return Err(Error::Library(name));
    }

    let function_pointer = GetProcAddress(library, proc_name);
    if function_pointer.is_null() {
        return Err(Error::Proc {
            lib: name,
            proc,
        });
    }

    Ok(function_pointer)
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Failed to load {0}")]
    Library(&'static str),
    #[error("Failed to load {proc} from {lib}")]
    Proc {
        lib: &'static str,
        proc: Proc,
    },
}

#[derive(Debug, Clone)]
pub enum Proc {
    Name(&'static str),
    Ordinal(u16),
}
impl core::fmt::Display for Proc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Name(name) => name.fmt(f),
            Self::Ordinal(ordinal) => ordinal.fmt(f),
        }
    }
}