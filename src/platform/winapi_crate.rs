use crate::{DllHandle, Error, WindowsDll, WindowsDllProc};

use core::mem::transmute;

pub(crate) use winapi::shared::{
    basetsd::ULONG_PTR,
    minwindef::{DWORD, HMODULE, WORD},
};
pub use winapi::um::winnt::{LPCSTR, LPCWSTR};
use winapi::{
    shared::minwindef::TRUE,
    um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryExW},
};

pub mod flags {
    pub const NO_FLAGS: LOAD_LIBRARY_FLAGS = 0;

    #[allow(non_camel_case_types)]
    pub type LOAD_LIBRARY_FLAGS = super::DWORD;

    pub use winapi::um::libloaderapi::{
        DONT_RESOLVE_DLL_REFERENCES, LOAD_IGNORE_CODE_AUTHZ_LEVEL, LOAD_LIBRARY_AS_DATAFILE,
        LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE, LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        LOAD_LIBRARY_REQUIRE_SIGNED_TARGET, LOAD_LIBRARY_SAFE_CURRENT_DIRS,
        LOAD_LIBRARY_SEARCH_APPLICATION_DIR, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32,
        LOAD_LIBRARY_SEARCH_USER_DIRS, LOAD_WITH_ALTERED_SEARCH_PATH,
    };
}

pub(crate) unsafe fn load_lib<T: WindowsDll>() -> DllHandle {
    let h_file = std::ptr::null_mut();

    DllHandle(LoadLibraryExW(T::LIB_LPCWSTR, h_file, T::FLAGS))
}
pub(crate) unsafe fn free_lib(library: DllHandle) -> bool {
    let succeeded = FreeLibrary(library.0);
    succeeded == TRUE
}
pub(crate) unsafe fn get_proc<T: WindowsDllProc>(library: DllHandle) -> Result<T::Sig, Error<T>> {
    let proc = GetProcAddress(library.0, T::PROC_LPCSTR as _);

    if proc.is_null() {
        Err(Error::proc())
    } else {
        Ok(*transmute::<_, &T::Sig>(&proc))
    }
}
