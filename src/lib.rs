#[doc(hidden)]
pub mod macro_internal;
mod platform;

pub use platform::{flags, DllCache, DllHandle, LPCSTR, LPCWSTR};
pub use windows_dll_codegen::dll;

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

pub trait WindowsDll: Sized {
    const LIB: &'static str;

    const LIB_LPCWSTR: LPCWSTR;
    const FLAGS: flags::LOAD_LIBRARY_FLAGS;

    unsafe fn cache() -> &'static DllCache<Self>;

    unsafe fn free() -> bool
    where
        Self: 'static,
    {
        let library = Self::cache();
        library.free_lib()
    }
}

pub trait WindowsDllProc: Sized {
    type Dll: WindowsDll + 'static;
    type Sig: Copy;
    const PROC: Proc;
    const PROC_LPCSTR: LPCSTR;

    unsafe fn proc() -> Result<Self::Sig, Error<Self>>;
    unsafe fn load() -> Result<Self::Sig, Error<Self>> {
        Self::Dll::cache().get_proc::<Self>()
    }
    unsafe fn exists() -> bool {
        Self::proc().is_ok()
    }
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
