use crate::{Error, ErrorKind, WindowsDll, WindowsDllProc};

use core::{
    marker::PhantomData,
    mem::transmute,
    sync::atomic::{AtomicIsize, AtomicUsize, Ordering},
};
use once_cell::sync::OnceCell;

pub(crate) use windows::Win32::Foundation::HINSTANCE;
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::{FARPROC, HANDLE},
        System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryExW},
    },
};

type NonNullFarProc = unsafe extern "system" fn() -> isize;
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

pub struct DllCache<D> {
    handle: AtomicIsize,
    procs: OnceCell<Vec<DllProcCache>>,
    _phantom: PhantomData<D>,
}
impl<D> DllCache<D> {
    pub const fn empty() -> Self {
        Self {
            handle: AtomicIsize::new(0),
            procs: OnceCell::new(),
            _phantom: PhantomData,
        }
    }
    fn load_handle(&self) -> HINSTANCE {
        HINSTANCE(self.handle.load(Ordering::SeqCst))
    }
    fn store_handle(&self, handle: HINSTANCE) {
        self.handle.store(handle.0, Ordering::SeqCst);
    }
    pub(crate) unsafe fn free_lib(&self) -> bool {
        let handle = self.load_handle();
        if handle.is_invalid() {
            false
        } else {
            self.store_handle(HINSTANCE(0));
            for proc in self.procs.get().into_iter().flatten() {
                proc.store_ptr(None);
            }

            let succeeded = FreeLibrary(handle);

            succeeded.as_bool()
        }
    }
}

impl<D: WindowsDll> DllCache<D> {
    unsafe fn get(&self) -> HINSTANCE {
        let handle = self.load_handle();

        let handle = if handle.0 == 0 {
            self.load_and_cache_lib()
        } else {
            handle
        };

        handle
    }
    unsafe fn load_and_cache_lib(&self) -> HINSTANCE {
        let handle = LoadLibraryExW(PCWSTR(D::LIB_LPCWSTR), HANDLE(0), D::FLAGS);

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
    ) -> Result<NonNullFarProc, ErrorKind> {
        let library = self.get();
        if library.is_invalid() {
            return Err(ErrorKind::Lib);
        }

        let cached_proc = &self.procs.get().unwrap()[cache_index];
        let cached_proc_ptr = cached_proc.load_ptr();

        cached_proc_ptr
            .or_else(|| GetProcAddress(library, PCSTR(name)))
            .ok_or(ErrorKind::Proc)
    }
    pub unsafe fn get_proc<P: WindowsDllProc<Dll = D>>(&self) -> Result<P::Sig, Error<P>> {
        let proc = self.get_proc_ptr(P::PROC_LPCSTR, P::CACHE_INDEX)?;
        Ok(*transmute::<_, &P::Sig>(&proc))
    }
}

struct DllProcCache(AtomicUsize);
impl DllProcCache {
    const fn empty() -> Self {
        Self(AtomicUsize::new(0))
    }
    unsafe fn load_ptr(&self) -> FARPROC {
        transmute(self.0.load(Ordering::SeqCst))
    }
    unsafe fn store_ptr(&self, handle: FARPROC) {
        self.0.store(transmute(handle), Ordering::SeqCst);
    }
}
