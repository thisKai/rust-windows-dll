use crate::{Error, WindowsDll, WindowsDllProc};

use core::{
    marker::PhantomData,
    mem::transmute,
    sync::atomic::{AtomicIsize, Ordering},
};

pub(crate) use windows::Win32::Foundation::HINSTANCE as HMODULE;
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::HANDLE,
        System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryExW},
    },
};

#[allow(non_camel_case_types)]
pub(crate) type ULONG_PTR = usize;
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

#[derive(Clone, Copy)]
pub struct DllHandle(HMODULE);
impl DllHandle {
    pub(crate) fn is_null(&self) -> bool {
        self.0.is_invalid()
    }
}

pub struct DllCache<D> {
    handle: AtomicIsize,
    _phantom: PhantomData<D>,
}
impl<D> DllCache<D> {
    pub const fn empty() -> Self {
        Self {
            handle: AtomicIsize::new(0),
            _phantom: PhantomData,
        }
    }
    fn load_handle(&self) -> HMODULE {
        HMODULE(self.handle.load(Ordering::SeqCst))
    }
    fn store_handle(&self, handle: HMODULE) {
        self.handle.store(handle.0, Ordering::SeqCst);
    }
    pub unsafe fn free_lib(&self) -> bool {
        let handle = self.load_handle();
        if handle.is_invalid() {
            false
        } else {
            let succeeded = FreeLibrary(handle);

            self.store_handle(HMODULE(0));

            succeeded.as_bool()
        }
    }
}

impl<D: WindowsDll> DllCache<D> {
    pub unsafe fn get(&self) -> DllHandle {
        let handle = self.handle.load(Ordering::SeqCst);

        let handle = if handle == 0 {
            self.load_and_cache_lib()
        } else {
            HMODULE(handle)
        };

        DllHandle(handle)
    }
    unsafe fn load_and_cache_lib(&self) -> HMODULE {
        let handle = LoadLibraryExW(PCWSTR(D::LIB_LPCWSTR), HANDLE(0), D::FLAGS);

        self.handle.store(handle.0, Ordering::SeqCst);

        handle
    }
    pub(crate) unsafe fn get_proc<P: WindowsDllProc<Dll = D>>(&self) -> Result<P::Sig, Error<P>> {
        let library = self.get();
        if library.is_null() {
            return Err(Error::lib());
        }
        GetProcAddress(library.0, PCSTR(P::PROC_LPCSTR))
            .map(|proc| *transmute::<_, &P::Sig>(&proc))
            .ok_or_else(Error::proc)
    }
}
