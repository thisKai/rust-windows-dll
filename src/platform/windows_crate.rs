use crate::{DllHandle, Error, WindowsDll, WindowsDllProc};

use core::mem::transmute;

pub(crate) use windows::Win32::Foundation::HINSTANCE as HMODULE;
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::HANDLE,
        System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryExW},
    },
};

pub(crate) type ULONG_PTR = usize;
pub(crate) type DWORD = u32;
pub(crate) type WORD = u16;
pub type LPCWSTR = *const u16;
pub type LPCSTR = *const u8;

pub mod flags {
    pub const NO_FLAGS: LOAD_LIBRARY_FLAGS = LOAD_LIBRARY_FLAGS(0);

    pub use windows::Win32::System::LibraryLoader::{
        DONT_RESOLVE_DLL_REFERENCES, LOAD_IGNORE_CODE_AUTHZ_LEVEL, LOAD_LIBRARY_AS_DATAFILE,
        LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE, LOAD_LIBRARY_AS_IMAGE_RESOURCE, LOAD_LIBRARY_FLAGS,
        LOAD_LIBRARY_OS_INTEGRITY_CONTINUITY, LOAD_LIBRARY_REQUIRE_SIGNED_TARGET,
        LOAD_LIBRARY_SAFE_CURRENT_DIRS, LOAD_LIBRARY_SEARCH_APPLICATION_DIR,
        LOAD_LIBRARY_SEARCH_DEFAULT_DIRS, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
        LOAD_LIBRARY_SEARCH_SYSTEM32, LOAD_LIBRARY_SEARCH_SYSTEM32_NO_FORWARDER,
        LOAD_LIBRARY_SEARCH_USER_DIRS, LOAD_WITH_ALTERED_SEARCH_PATH,
    };
}

pub(crate) unsafe fn load_lib<T: WindowsDll>() -> DllHandle {
    let h_file = HANDLE(0);

    DllHandle(LoadLibraryExW(PCWSTR(T::LIB_LPCWSTR), h_file, T::FLAGS))
}
pub(crate) unsafe fn free_lib(library: DllHandle) -> bool {
    FreeLibrary(library.0).as_bool()
}
pub(crate) unsafe fn get_proc<T: WindowsDllProc>(library: DllHandle) -> Result<T::Sig, Error<T>> {
    GetProcAddress(library.0, PCSTR(T::PROC_LPCSTR))
        .map(|proc| *transmute::<_, &T::Sig>(&proc))
        .ok_or_else(Error::proc)
}
