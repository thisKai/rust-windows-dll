use crate::{platform, LPCSTR};
pub use core::{self, option::Option, result::Result};
pub use platform::DllCache;

// Copied MAKEINTRESOURCEA function from winapi so that it can be const
#[inline]
pub const fn make_int_resource_a(i: platform::WORD) -> LPCSTR {
    i as platform::ULONG_PTR as _
}
