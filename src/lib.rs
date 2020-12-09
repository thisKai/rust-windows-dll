pub use {
    core,
    core::result::Result,
    once_cell,
    winapi::{
        shared::minwindef::DWORD,
        um::winnt::{LPCSTR, LPCWSTR},
    },
    windows_dll_codegen::dll,
};

use {
    core::marker::PhantomData,
    winapi::shared::{
        basetsd::ULONG_PTR,
        minwindef::{FARPROC, WORD},
    },
};

#[inline]
pub unsafe fn load_dll_proc<D: DllProc>() -> Result<FARPROC, Error<D>> {
    use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryExW};

    let library = LoadLibraryExW(D::LIB_LPCWSTR, std::ptr::null_mut(), D::FLAGS);
    if library.is_null() {
        return Err(Error::lib());
    }

    let function_pointer = GetProcAddress(library, D::PROC_LPCSTR);
    if function_pointer.is_null() {
        return Err(Error::proc());
    }

    Ok(function_pointer)
}

pub mod flags {
    pub use winapi::um::libloaderapi::{
        DONT_RESOLVE_DLL_REFERENCES, LOAD_IGNORE_CODE_AUTHZ_LEVEL, LOAD_LIBRARY_AS_DATAFILE,
        LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE, LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        LOAD_LIBRARY_REQUIRE_SIGNED_TARGET, LOAD_LIBRARY_SAFE_CURRENT_DIRS,
        LOAD_LIBRARY_SEARCH_APPLICATION_DIR, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32,
        LOAD_LIBRARY_SEARCH_USER_DIRS, LOAD_WITH_ALTERED_SEARCH_PATH,
    };
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

pub trait DllProc {
    const LIB: &'static str;
    const LIB_LPCWSTR: LPCWSTR;
    const PROC: Proc;
    const PROC_LPCSTR: LPCSTR;
    const FLAGS: DWORD;
}

/// Copied MAKEINTRESOURCEA function from winapi so that it can be const
#[inline]
pub const fn make_int_resource_a(i: WORD) -> LPCSTR {
    i as ULONG_PTR as _
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum ErrorKind {
    Lib,
    Proc,
}

pub struct Error<D: DllProc> {
    pub kind: ErrorKind,
    _dll: PhantomData<D>,
}
impl<D: DllProc> core::fmt::Debug for Error<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("lib", &D::LIB)
            .field("proc", &D::PROC)
            .finish()
    }
}
impl<D: DllProc> Clone for Error<D> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<D: DllProc> Copy for Error<D> {}

impl<D: DllProc> Error<D> {
    pub fn lib() -> Self {
        Self {
            kind: ErrorKind::Lib,
            _dll: PhantomData,
        }
    }
    pub fn proc() -> Self {
        Self {
            kind: ErrorKind::Proc,
            _dll: PhantomData,
        }
    }
}
impl<D: DllProc> core::fmt::Display for Error<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.kind {
            ErrorKind::Lib => write!(f, "Could not load {}", D::LIB),
            ErrorKind::Proc => write!(f, "Could not load {}#{}", D::LIB, D::PROC),
        }
    }
}
impl<D: DllProc> std::error::Error for Error<D> {}
