use crate::{platform, Error, WindowsDllProc, LPCSTR};
pub use core::{self, option::Option, result::Result};
use once_cell::sync::Lazy;
pub use platform::DllCache;

pub type DllProcCache<D> = Lazy<Result<<D as WindowsDllProc>::Sig, Error<D>>>;

// Copied MAKEINTRESOURCEA function from winapi so that it can be const
#[inline]
pub const fn make_int_resource_a(i: platform::WORD) -> LPCSTR {
    i as platform::ULONG_PTR as _
}
