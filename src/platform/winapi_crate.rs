use crate::{Error, ErrorKind, WindowsDll, WindowsDllProc};

use core::{
    marker::PhantomData,
    mem::transmute,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};
use once_cell::sync::OnceCell;

pub(crate) use winapi::shared::{
    basetsd::ULONG_PTR,
    minwindef::{DWORD, HMODULE, WORD},
};
pub use winapi::um::winnt::{LPCSTR, LPCWSTR};
use winapi::{
    shared::minwindef::{__some_function, FARPROC, HINSTANCE__, TRUE},
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

#[doc(hidden)]
pub struct DllCache<D> {
    handle: AtomicPtr<HINSTANCE__>,
    procs: OnceCell<Vec<DllProcCache>>,
    _phantom: PhantomData<D>,
}
impl<D> DllCache<D> {
    pub const fn empty() -> Self {
        Self {
            handle: AtomicPtr::new(ptr::null_mut()),
            procs: OnceCell::new(),
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
            for proc in self.procs.get().into_iter().flatten() {
                proc.store_ptr(ptr::null_mut());
            }

            let succeeded = FreeLibrary(self.load_handle());

            succeeded == TRUE
        }
    }
}

impl<D: WindowsDll> DllCache<D> {
    pub(crate) unsafe fn lib_exists(&self) -> bool {
        !self.get().is_null()
    }
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
        self.procs.get_or_init(|| {
            let mut procs = Vec::with_capacity(D::LEN);
            for _ in 0..D::LEN {
                procs.push(DllProcCache::empty());
            }
            procs
        });

        handle
    }
    unsafe fn get_proc_ptr(
        &self,
        name: LPCSTR,
        cache_index: usize,
    ) -> Result<FARPROC, ErrorKind> {
        let library = self.get();
        if library.is_null() {
            return Err(ErrorKind::Lib);
        }

        let cached_proc = &self.procs.get().unwrap()[cache_index];
        let cached_proc_ptr = cached_proc.load_ptr();

        let proc = if cached_proc_ptr.is_null() {
            let proc = GetProcAddress(library, name as _);
            if proc.is_null() {
                return Err(ErrorKind::Proc);
            }
            proc
        } else {
            cached_proc_ptr
        };

        Ok(proc)
    }
    pub unsafe fn get_proc<P: WindowsDllProc<Dll = D>>(&self) -> Result<P::Sig, Error<P>> {
        let proc = self.get_proc_ptr(P::PROC_LPCSTR, P::CACHE_INDEX)?;
        Ok(*transmute::<_, &P::Sig>(&proc))
    }
}

struct DllProcCache(AtomicPtr<__some_function>);
impl DllProcCache {
    const fn empty() -> Self {
        Self(AtomicPtr::new(ptr::null_mut()))
    }
    fn load_ptr(&self) -> FARPROC {
        self.0.load(Ordering::SeqCst)
    }
    fn store_ptr(&self, handle: FARPROC) {
        self.0.store(handle, Ordering::SeqCst);
    }
}
