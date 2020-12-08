pub use {
    windows_dll_codegen::dll,
    once_cell,
    core,
    core::result::Result,
};

use winapi::{
    shared::{
        minwindef::{WORD, FARPROC, DWORD},
    },
    um::winnt::{LPCSTR, LPCWSTR},
    um::winuser::MAKEINTRESOURCEA,
};

#[doc(hidden)]
pub mod load {
    use super::*;

    #[doc(hidden)]
    #[inline]
    pub unsafe fn load_dll_proc_ordinal(name: &'static str, proc: Proc, name_lpcwstr: LPCWSTR, proc_ordinal: WORD, flags: DWORD) -> Result<FARPROC, Error> {
        load_dll_proc_name(name, proc, name_lpcwstr, MAKEINTRESOURCEA(proc_ordinal), flags)
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn load_dll_proc_name(name: &'static str, proc: Proc, name_lpcwstr: LPCWSTR, proc_name: LPCSTR, flags: DWORD) -> Result<FARPROC, Error> {
        use winapi::um::libloaderapi::{LoadLibraryExW, GetProcAddress};

        let library = LoadLibraryExW(name_lpcwstr, std::ptr::null_mut(), flags);
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
}

pub mod flags {
    pub use winapi::um::libloaderapi::{
        DONT_RESOLVE_DLL_REFERENCES,
        LOAD_IGNORE_CODE_AUTHZ_LEVEL,
        LOAD_LIBRARY_AS_DATAFILE,
        LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE,
        LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        LOAD_LIBRARY_SEARCH_APPLICATION_DIR,
        LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
        LOAD_LIBRARY_SEARCH_SYSTEM32,
        LOAD_LIBRARY_SEARCH_USER_DIRS,
        LOAD_WITH_ALTERED_SEARCH_PATH,
        LOAD_LIBRARY_REQUIRE_SIGNED_TARGET,
        LOAD_LIBRARY_SAFE_CURRENT_DIRS,
    };
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
