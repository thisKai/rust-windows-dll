mod platform;
pub use platform::{flags, LPCSTR, LPCWSTR};
pub use windows_dll_codegen::dll;
#[doc(hidden)]
pub use {
    core::{self, option::Option, result::Result},
    once_cell,
};

use core::marker::PhantomData;

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
pub struct DllHandle(platform::HMODULE);
impl DllHandle {
    #[cfg(feature = "winapi")]
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
    #[cfg(feature = "windows")]
    fn is_null(&self) -> bool {
        self.0 .0 == 0
    }
}
unsafe impl Send for DllHandle {}
unsafe impl Sync for DllHandle {}

pub trait WindowsDll: Sized {
    const LIB: &'static str;

    const LIB_LPCWSTR: LPCWSTR;
    const FLAGS: flags::LOAD_LIBRARY_FLAGS;

    unsafe fn ptr() -> DllHandle;

    unsafe fn load() -> DllHandle {
        platform::load_lib::<Self>()
    }

    unsafe fn free() -> bool {
        let library = Self::ptr();
        if library.is_null() {
            false
        } else {
            platform::free_lib(library)
        }
    }
}

pub trait WindowsDllProc: Sized {
    type Dll: WindowsDll;
    type Sig: Copy;
    const PROC: Proc;
    const PROC_LPCSTR: LPCSTR;

    unsafe fn proc() -> Result<Self::Sig, Error<Self>>;
    unsafe fn load() -> Result<Self::Sig, Error<Self>> {
        let library = Self::Dll::ptr();

        if library.is_null() {
            Err(Error::lib())
        } else {
            platform::get_proc::<Self>(library)
        }
    }
    unsafe fn exists() -> bool {
        Self::proc().is_ok()
    }
}


// Copied MAKEINTRESOURCEA function from winapi so that it can be const
#[doc(hidden)]
#[inline]
pub const fn make_int_resource_a(i: platform::WORD) -> LPCSTR {
    i as platform::ULONG_PTR as _
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
