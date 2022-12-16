use std::error::Error;
use windows_dll::dll;

use self::platform::*;

#[dll(ntdll)]
extern "system" {
    #[allow(non_snake_case)]
    #[fallible]
    fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOW) -> NTSTATUS;
}

#[allow(non_snake_case)]
#[repr(C)]
struct OSVERSIONINFOW {
    dwOSVersionInfoSize: ULONG,
    dwMajorVersion: ULONG,
    dwMinorVersion: ULONG,
    dwBuildNumber: ULONG,
    dwPlatformId: ULONG,
    szCSDVersion: [WCHAR; 128],
}

#[cfg(feature = "winapi")]
mod platform {
    use super::*;
    use winapi::shared::ntdef::NT_SUCCESS;
    pub use winapi::shared::{
        minwindef::ULONG,
        ntdef::{NTSTATUS, WCHAR},
    };

    #[test]
    fn propagate_errors() -> Result<(), Box<dyn Error>> {
        unsafe {
            let mut vi = OSVERSIONINFOW {
                dwOSVersionInfoSize: 0,
                dwMajorVersion: 0,
                dwMinorVersion: 0,
                dwBuildNumber: 0,
                dwPlatformId: 0,
                szCSDVersion: [0; 128],
            };

            let status = RtlGetVersion(&mut vi as _)?;

            if NT_SUCCESS(status) {
                dbg!(vi.dwBuildNumber);
                Ok(())
            } else {
                Err("RtlGetVersion error")?
            }
        }
    }
}

#[cfg(feature = "windows")]
mod platform {
    use super::*;
    pub use windows::Win32::Foundation::NTSTATUS;

    pub type ULONG = u32;
    pub type WCHAR = u16;

    #[test]
    fn propagate_errors() -> Result<(), Box<dyn Error>> {
        unsafe {
            let mut vi = OSVERSIONINFOW {
                dwOSVersionInfoSize: 0,
                dwMajorVersion: 0,
                dwMinorVersion: 0,
                dwBuildNumber: 0,
                dwPlatformId: 0,
                szCSDVersion: [0; 128],
            };

            let status = RtlGetVersion(&mut vi as _)?;

            if status.is_ok() {
                dbg!(vi.dwBuildNumber);
                Ok(())
            } else {
                Err("RtlGetVersion error")?
            }
        }
    }
}

#[cfg(feature = "windows-sys")]
mod platform {
    use super::*;
    pub use windows_sys::Win32::Foundation::NTSTATUS;

    pub type ULONG = u32;
    pub type WCHAR = u16;

    #[allow(non_snake_case)]
    #[inline]
    fn NT_SUCCESS(status: NTSTATUS) -> bool {
        status >= 0
    }

    #[test]
    fn propagate_errors() -> Result<(), Box<dyn Error>> {
        unsafe {
            let mut vi = OSVERSIONINFOW {
                dwOSVersionInfoSize: 0,
                dwMajorVersion: 0,
                dwMinorVersion: 0,
                dwBuildNumber: 0,
                dwPlatformId: 0,
                szCSDVersion: [0; 128],
            };

            let status = RtlGetVersion(&mut vi as _)?;

            if NT_SUCCESS(status) {
                dbg!(vi.dwBuildNumber);
                Ok(())
            } else {
                Err("RtlGetVersion error")?
            }
        }
    }
}
