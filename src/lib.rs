pub use {
    windows_dll_codegen::dll,
    once_cell,
    core,
    core::result::Result,
    winapi::um::winnt::{LPCSTR, LPCWSTR},
};

use winapi::shared::{
    minwindef::{WORD, FARPROC},
    basetsd::ULONG_PTR,
};


#[inline]
pub unsafe fn load_dll_proc<D: Dll>() -> Result<FARPROC, Error> {
    use winapi::um::libloaderapi::{LoadLibraryW, GetProcAddress};

    let library = LoadLibraryW(D::LIB_LPCWSTR);
    if library.is_null() {
        return Err(Error::Library(D::LIB));
    }

    let function_pointer = GetProcAddress(library, D::PROC_LPCSTR);
    if function_pointer.is_null() {
        return Err(Error::Proc {
            lib: D::LIB,
            proc: D::PROC,
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

pub trait Dll: Sized {
    const LIB: &'static str;
    const LIB_LPCWSTR: LPCWSTR;
    const PROC: Proc;
    const PROC_LPCSTR: LPCSTR;
}

#[inline]
pub const fn make_int_resource_a(i: WORD) -> LPCSTR {
    i as ULONG_PTR as _
}