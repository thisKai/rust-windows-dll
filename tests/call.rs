#[cfg(feature = "winapi")]
mod winapi {
    use {
        std::error::Error,
        winapi::shared::{
            minwindef::ULONG,
            ntdef::{NTSTATUS, NT_SUCCESS, WCHAR},
        },
        windows_dll::dll,
    };

    #[dll(ntdll)]
    extern "system" {
        #[allow(non_snake_case)]
        #[fallible]
        fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOW) -> NTSTATUS;
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
}
