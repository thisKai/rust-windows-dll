use crate::{
    platform::{AtomicDllHandle, AtomicDllProcPtr, DllHandle, DllProcPtr, LPCSTR},
    Error, ErrorKind, WindowsDll, WindowsDllProc,
};
use core::marker::PhantomData;
use once_cell::sync::OnceCell;

#[doc(hidden)]
pub struct DllCache<D> {
    handle: AtomicDllHandle,
    procs: OnceCell<Vec<AtomicDllProcPtr>>,
    _phantom: PhantomData<D>,
}
impl<D> DllCache<D> {
    pub const fn empty() -> Self {
        Self {
            handle: AtomicDllHandle::empty(),
            procs: OnceCell::new(),
            _phantom: PhantomData,
        }
    }
    pub(crate) unsafe fn free_lib(&self) -> bool {
        let handle = self.handle.load();
        if handle.is_invalid() {
            false
        } else {
            self.handle.clear();
            for proc in self.procs.get().into_iter().flatten() {
                proc.store(None);
            }

            handle.free()
        }
    }
}

impl<D: WindowsDll> DllCache<D> {
    pub(crate) unsafe fn lib_exists(&self) -> bool {
        !self.get().is_invalid()
    }
    unsafe fn get(&self) -> DllHandle {
        let handle = self.handle.load();

        let handle = if handle.is_invalid() {
            self.load_and_cache_lib()
        } else {
            handle
        };

        handle
    }
    unsafe fn load_and_cache_lib(&self) -> DllHandle {
        let handle = DllHandle::load(D::LIB_LPCWSTR, D::FLAGS);

        self.procs.get_or_init(|| {
            let mut procs = Vec::with_capacity(D::LEN);
            for _ in 0..D::LEN {
                procs.push(AtomicDllProcPtr::empty());
            }
            procs
        });
        // Store the handle *after* initializing `self.procs`. This
        // is required to avoid a race condition in `get_proc_ptr`.
        self.handle.store(handle);

        handle
    }
    unsafe fn get_proc_ptr(
        &self,
        name: LPCSTR,
        cache_index: usize,
    ) -> Result<DllProcPtr, ErrorKind> {
        let library = self.get();
        if library.is_invalid() {
            return Err(ErrorKind::Lib);
        }

        // The unwrap is safe because `self.procs` is guaranteed to
        // be initialized *before* `self.handle` is set. See
        // `load_and_cache_lib`.
        let cached_proc = &self.procs.get().unwrap()[cache_index];

        cached_proc
            .load()
            .or_else(|| library.get_proc(name))
            .ok_or(ErrorKind::Proc)
    }
    pub unsafe fn get_proc<P: WindowsDllProc<Dll = D>>(&self) -> Result<P::Sig, Error<P>> {
        let proc = self.get_proc_ptr(P::PROC_LPCSTR, P::CACHE_INDEX)?;
        Ok(proc.transmute())
    }
}
