#[cfg(not(feature = "windows"))]
fn main() {}

#[cfg(feature = "windows")]
fn main() {
    windows::build!(
        windows::win32::system_services::{
            LoadLibraryExW,
            FreeLibrary,
            GetProcAddress,
        }
    );
}
