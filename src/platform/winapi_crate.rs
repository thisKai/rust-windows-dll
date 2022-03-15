use crate::{Error, WindowsDll, WindowsDllProc};

use core::{
    mem::transmute,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};
use std::marker::PhantomData;

pub(crate) use winapi::shared::{
    basetsd::ULONG_PTR,
    minwindef::{DWORD, HMODULE, WORD},
};
pub use winapi::um::winnt::{LPCSTR, LPCWSTR};
use winapi::{
    shared::minwindef::{HINSTANCE__, TRUE},
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

pub struct DllCache<D> {
    handle: AtomicPtr<HINSTANCE__>,
    _phantom: PhantomData<D>,
}
impl<D> DllCache<D> {
    pub const fn empty() -> Self {
        Self {
            handle: AtomicPtr::new(ptr::null_mut()),
            _phantom: PhantomData,
        }
    }
    fn load_handle(&self) -> HMODULE {
        self.handle.load(Ordering::SeqCst)
    }
    fn store_handle(&self, handle: HMODULE) {
        self.handle.store(handle, Ordering::SeqCst);
    }
    pub(crate) unsafe fn free_lib(&self) -> bool {
        let handle = self.load_handle();
        if handle.is_null() {
            false
        } else {
            self.store_handle(ptr::null_mut());

            let succeeded = FreeLibrary(self.load_handle());

            succeeded == TRUE
        }
    }
}

impl<D: WindowsDll> DllCache<D> {
    unsafe fn get(&self) -> HMODULE {
        let handle = self.load_handle();

        let handle = if handle.is_null() {
            self.load_and_cache_lib()
        } else {
            handle
        };

        handle
    }
    unsafe fn load_and_cache_lib(&self) -> HMODULE {
        let handle = LoadLibraryExW(D::LIB_LPCWSTR, ptr::null_mut(), D::FLAGS);

        self.store_handle(handle);

        handle
    }
    pub(crate) unsafe fn get_proc<P: WindowsDllProc<Dll = D>>(&self) -> Result<P::Sig, Error<P>> {
        let library = self.get();
        if library.is_null() {
            return Err(Error::lib());
        }
        let proc = GetProcAddress(library, P::PROC_LPCSTR as _);
        if proc.is_null() {
            return Err(Error::proc());
        }
        Ok(*transmute::<_, &P::Sig>(&proc))
    }
}
