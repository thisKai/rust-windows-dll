pub use windows_dll_codegen::dll;
#[doc(hidden)]
pub use {
    core::{self, option::Option, result::Result},
    once_cell,
};

#[doc(hidden)]
#[cfg(feature = "winapi")]
pub use winapi::{
    shared::minwindef::{DWORD, FALSE, FARPROC, TRUE},
    um::winnt::{LPCSTR, LPCWSTR},
};

use core::marker::PhantomData;

#[cfg(feature = "winapi")]
use winapi::{
    shared::{
        basetsd::ULONG_PTR,
        minwindef::{HMODULE, WORD},
    },
    um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryExW},
};

#[cfg(feature = "winapi")]
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

#[cfg(feature = "windows")]
mod windows_rs {
    ::windows::include_bindings!();
    #[allow(non_camel_case_types)]
    pub type ULONG_PTR = usize;
    pub type HMODULE = isize;
    pub type DWORD = u32;
    pub type WORD = u16;
    pub type LPCWSTR = *const u16;
    pub type LPCSTR = *const i8;
}
#[cfg(feature = "windows")]
use windows_rs::{
    windows::win32::system_services::{
        FreeLibrary, GetProcAddress, LoadLibraryExW, FARPROC, HANDLE,
    },
    HMODULE, ULONG_PTR, WORD,
};
#[cfg(feature = "windows")]
pub use windows_rs::{DWORD, LPCSTR, LPCWSTR};

#[cfg(feature = "windows")]
pub mod flags {
    pub const DONT_RESOLVE_DLL_REFERENCES: u32 = 0x00000001;
    pub const LOAD_LIBRARY_AS_DATAFILE: u32 = 0x00000002;
    pub const LOAD_WITH_ALTERED_SEARCH_PATH: u32 = 0x00000008;
    pub const LOAD_IGNORE_CODE_AUTHZ_LEVEL: u32 = 0x00000010;
    pub const LOAD_LIBRARY_AS_IMAGE_RESOURCE: u32 = 0x00000020;
    pub const LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE: u32 = 0x00000040;
    pub const LOAD_LIBRARY_REQUIRE_SIGNED_TARGET: u32 = 0x00000080;
    pub const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: u32 = 0x00000100;
    pub const LOAD_LIBRARY_SEARCH_APPLICATION_DIR: u32 = 0x00000200;
    pub const LOAD_LIBRARY_SEARCH_USER_DIRS: u32 = 0x00000400;
    pub const LOAD_LIBRARY_SEARCH_SYSTEM32: u32 = 0x00000800;
    pub const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: u32 = 0x00001000;
    pub const LOAD_LIBRARY_SAFE_CURRENT_DIRS: u32 = 0x00002000;
    pub const LOAD_LIBRARY_SEARCH_SYSTEM32_NO_FORWARDER: u32 = 0x00004000;
    pub const LOAD_LIBRARY_OS_INTEGRITY_CONTINUITY: u32 = 0x00008000;
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
    #[cfg(feature = "winapi")]
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
    #[cfg(feature = "windows")]
    fn is_null(&self) -> bool {
        self.0 == 0
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
        #[cfg(feature = "winapi")]
        let h_file = std::ptr::null_mut();

        #[cfg(feature = "windows")]
        let h_file = HANDLE(0);

        DllHandle(LoadLibraryExW(Self::LIB_LPCWSTR, h_file, Self::FLAGS))
    }
    #[cfg(feature = "winapi")]
    unsafe fn free() -> bool {
        let library = Self::ptr();
        if library.is_null() {
            false
        } else {
            let succeeded = FreeLibrary(library.0);
            succeeded == TRUE
        }
    }
    #[cfg(feature = "windows")]
    unsafe fn free() -> bool {
        let library = Self::ptr();
        if library.is_null() {
            false
        } else {
            FreeLibrary(library.0).is_ok()
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
        let library = Self::Dll::ptr();

        if library.is_null() {
            Err(Error::lib())
        } else {
            let proc = GetProcAddress(library.0, Self::PROC_LPCSTR);

            #[cfg(feature = "winapi")]
            if proc.is_null() {
                Err(Error::proc())
            } else {
                Ok(proc)
            }

            #[cfg(feature = "windows")]
            proc.ok_or(Error::proc())
        }
    }
    unsafe fn exists() -> bool {
        Self::proc().is_ok()
    }
}

// Copied MAKEINTRESOURCEA function from winapi so that it can be const
#[doc(hidden)]
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
