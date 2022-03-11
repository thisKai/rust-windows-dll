#[cfg(feature = "winapi")]
mod winapi_crate;
#[cfg(feature = "winapi")]
pub use winapi_crate::*;

#[cfg(feature = "windows")]
mod windows_crate;
#[cfg(feature = "windows")]
pub use windows_crate::*;
