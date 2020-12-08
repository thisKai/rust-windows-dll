pub use {
    windows_dll_codegen::dll,
    once_cell,
    core,
    core::result::Result,
    winapi::um::winnt::{LPCSTR, LPCWSTR},
};

use {
    core::marker::PhantomData,
    winapi::shared::{
        minwindef::{WORD, FARPROC},
        basetsd::ULONG_PTR,
    },
};


#[inline]
pub unsafe fn load_dll_proc<D: DllProc>() -> Result<FARPROC, Error<D>> {
    use winapi::um::libloaderapi::{LoadLibraryW, GetProcAddress};

    let library = LoadLibraryW(D::LIB_LPCWSTR);
    if library.is_null() {
        return Err(Error::lib());
    }

    let function_pointer = GetProcAddress(library, D::PROC_LPCSTR);
    if function_pointer.is_null() {
        return Err(Error::proc());
    }

    Ok(function_pointer)
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

pub trait DllProc: Sized + core::fmt::Debug {
    const LIB: &'static str;
    const LIB_LPCWSTR: LPCWSTR;
    const PROC: Proc;
    const PROC_LPCSTR: LPCSTR;
}

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

#[derive(Debug, Copy, Clone)]
pub struct Error<D: DllProc> {
    pub kind: ErrorKind,
    _dll: PhantomData<D>,
}
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