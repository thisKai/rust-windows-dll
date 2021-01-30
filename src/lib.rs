pub use {
    core::{self, option::Option, result::Result},
    once_cell,
    winapi::{
        shared::minwindef::{DWORD, FALSE, FARPROC, TRUE},
        um::winnt::{LPCSTR, LPCWSTR},
    },
    windows_dll_codegen::dll,
};

use {
    core::marker::PhantomData,
    winapi::shared::{
        basetsd::ULONG_PTR,
        minwindef::{HMODULE, WORD},
    },
};

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

#[derive(Clone, Copy)]
pub struct DllHandle(HMODULE);
impl DllHandle {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
unsafe impl Send for DllHandle {}
unsafe impl Sync for DllHandle {}

pub trait WindowsDll {
    const LIB: &'static str;

    const LIB_LPCWSTR: LPCWSTR;
    const FLAGS: DWORD;

    unsafe fn ptr() -> DllHandle;
    unsafe fn load() -> DllHandle {
        use winapi::um::libloaderapi::LoadLibraryExW;

        DllHandle(LoadLibraryExW(
            Self::LIB_LPCWSTR,
            std::ptr::null_mut(),
            Self::FLAGS,
        ))
    }
    unsafe fn free() -> bool {
        use winapi::um::libloaderapi::FreeLibrary;

        let library = Self::ptr();
        if library.is_null() {
            false
        } else {
            let succeeded = FreeLibrary(library.0);
            succeeded == TRUE
        }
    }
}

pub trait WindowsDllProc: Sized {
    type Dll: WindowsDll;
    type Sig;
    const PROC: Proc;
    const PROC_LPCSTR: LPCSTR;

    unsafe fn proc() -> Result<Self::Sig, Error<Self>>;
    unsafe fn load() -> Result<FARPROC, Error<Self>> {
        use winapi::um::libloaderapi::GetProcAddress;

        let library = Self::Dll::ptr();

        if library.is_null() {
            Err(Error::lib())
        } else {
            let proc = GetProcAddress(library.0, Self::PROC_LPCSTR);
            if proc.is_null() {
                Err(Error::proc())
            } else {
                Ok(proc)
            }
        }
    }
    unsafe fn exists() -> bool {
        Self::proc().is_ok()
    }
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

pub struct Error<D: WindowsDllProc> {
    pub kind: ErrorKind,
    _dll: PhantomData<D>,
}
impl<D: WindowsDllProc> core::fmt::Debug for Error<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("lib", &D::Dll::LIB)
            .field("proc", &D::PROC)
            .finish()
    }
}
impl<D: WindowsDllProc> Clone for Error<D> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<D: WindowsDllProc> Copy for Error<D> {}

impl<D: WindowsDllProc> Error<D> {
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
impl<D: WindowsDllProc> core::fmt::Display for Error<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.kind {
            ErrorKind::Lib => write!(f, "Could not load {}", D::Dll::LIB),
            ErrorKind::Proc => write!(f, "Could not load {}#{}", D::Dll::LIB, D::PROC),
        }
    }
}
impl<D: WindowsDllProc> std::error::Error for Error<D> {}
