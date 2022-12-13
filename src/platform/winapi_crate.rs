use core::{
    mem::transmute,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

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

#[repr(transparent)]
pub(crate) struct AtomicDllHandle(AtomicPtr<HINSTANCE__>);
impl AtomicDllHandle {
    pub(crate) const fn empty() -> Self {
        Self(AtomicPtr::new(ptr::null_mut()))
    }
    pub(crate) fn load(&self) -> DllHandle {
        DllHandle(self.0.load(Ordering::SeqCst))
    }
    pub(crate) fn store(&self, handle: DllHandle) {
        self.0.store(handle.0, Ordering::SeqCst);
    }
    pub(crate) fn clear(&self) {
        self.0.store(ptr::null_mut(), Ordering::SeqCst);
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct DllHandle(HMODULE);
impl DllHandle {
    pub(crate) unsafe fn load(
        lib_file_name: LPCWSTR,
        flags: flags::LOAD_LIBRARY_FLAGS,
    ) -> Option<Self> {
        Some(Self(LoadLibraryExW(lib_file_name, ptr::null_mut(), flags)))
    }
    pub(crate) fn is_invalid(&self) -> bool {
        self.0.is_null()
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
pub(crate) struct AtomicDllProcPtr(AtomicPtr<__some_function>);
impl AtomicDllProcPtr {
    pub(crate) const fn empty() -> Self {
        Self(AtomicPtr::new(ptr::null_mut()))
    }
    pub(crate) fn load(&self) -> Option<DllProcPtr> {
        DllProcPtr::new(self.0.load(Ordering::SeqCst))
    }
    pub(crate) fn store(&self, handle: Option<DllProcPtr>) {
        self.0.store(
            handle
                .map(|proc| proc.0.as_ptr())
                .unwrap_or(ptr::null_mut()),
            Ordering::SeqCst,
        );
    }
}

#[repr(transparent)]
pub(crate) struct DllProcPtr(ptr::NonNull<__some_function>);
impl DllProcPtr {
    fn new(proc: FARPROC) -> Option<Self> {
        ptr::NonNull::new(proc).map(DllProcPtr)
    }
    pub(crate) unsafe fn transmute<T: Copy>(self) -> T {
        *transmute::<_, &T>(&self.0)
    }
}
