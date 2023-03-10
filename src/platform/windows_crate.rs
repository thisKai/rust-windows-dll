use core::{
    mem::transmute,
    sync::atomic::{AtomicIsize, AtomicUsize, Ordering},
};

pub(crate) use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::{
    Foundation::{FARPROC, INVALID_HANDLE_VALUE, TRUE},
    System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryExW},
};

type NonNullFarProc = unsafe extern "system" fn() -> isize;
#[allow(non_camel_case_types)]
pub(crate) type ULONG_PTR = usize;
pub(crate) type WORD = u16;
pub type LPCWSTR = *const u16;
pub type LPCSTR = *const u8;

pub mod flags {
    pub const NO_FLAGS: LOAD_LIBRARY_FLAGS = 0;

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

#[repr(transparent)]
pub(crate) struct AtomicDllHandle(AtomicIsize);
impl AtomicDllHandle {
    pub(crate) const fn empty() -> Self {
        Self(AtomicIsize::new(0))
    }
    pub(crate) fn load(&self) -> DllHandle {
        DllHandle(self.0.load(Ordering::SeqCst))
    }
    pub(crate) fn store(&self, handle: DllHandle) {
        self.0.store(handle.0, Ordering::SeqCst);
    }
    pub(crate) fn clear(&self) {
        self.0.store(0, Ordering::SeqCst);
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct DllHandle(HINSTANCE);
impl DllHandle {
    pub(crate) unsafe fn load(lib_file_name: LPCWSTR, flags: flags::LOAD_LIBRARY_FLAGS) -> Self {
        Self(LoadLibraryExW(lib_file_name, 0, flags))
    }
    pub(crate) fn is_invalid(&self) -> bool {
        self.0 == INVALID_HANDLE_VALUE || self.0 == 0
    }
    pub(crate) unsafe fn free(self) -> bool {
        let succeeded = FreeLibrary(self.0);
        succeeded == TRUE
    }
    pub(crate) unsafe fn get_proc(&self, name: LPCSTR) -> Option<DllProcPtr> {
        DllProcPtr::new(GetProcAddress(self.0, name))
    }
}

#[repr(transparent)]
pub(crate) struct AtomicDllProcPtr(AtomicUsize);
impl AtomicDllProcPtr {
    pub(crate) const fn empty() -> Self {
        Self(AtomicUsize::new(0))
    }
    pub(crate) unsafe fn load(&self) -> Option<DllProcPtr> {
        DllProcPtr::new(transmute(self.0.load(Ordering::SeqCst)))
    }
    pub(crate) unsafe fn store(&self, handle: Option<DllProcPtr>) {
        self.0.store(
            handle.map(|proc| transmute(proc)).unwrap_or(0),
            Ordering::SeqCst,
        );
    }
}

#[repr(transparent)]
pub(crate) struct DllProcPtr(NonNullFarProc);
impl DllProcPtr {
    fn new(proc: FARPROC) -> Option<Self> {
        proc.map(DllProcPtr)
    }
    pub(crate) unsafe fn transmute<T: Copy>(self) -> T {
        *transmute::<_, &T>(&self.0)
    }
}
