# windows-dll
Macro for dynamically loading windows dll functions

## Usage
```rust
use {
    windows_dll::dll,
    winapi::shared::{
        minwindef::ULONG,
        ntdef::{NTSTATUS, NT_SUCCESS, WCHAR},
    },
};

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

#[dll("ntdll.dll")]
extern "system" {
    #[allow(non_snake_case)]
    fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOW) -> NTSTATUS;
}


fn os_version_info() -> OSVERSIONINFOW {
    unsafe {
        let mut vi = OSVERSIONINFOW {
            dwOSVersionInfoSize: 0,
            dwMajorVersion: 0,
            dwMinorVersion: 0,
            dwBuildNumber: 0,
            dwPlatformId: 0,
            szCSDVersion: [0; 128],
        };

        let status = RtlGetVersion(&mut vi as _);

        if NT_SUCCESS(status) {
            vi
        } else {
            panic!()
        }
    }
}
```

### Return a result to determine whether the function can be retrieved
```rust
#[dll("ntdll.dll")]
extern "system" {
    #[allow(non_snake_case)]
    #[fallible]
    fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOW) -> NTSTATUS;
}

fn os_version_info() -> Result<OSVERSIONINFOW, windows_dll::Error> {
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
            Ok(vi)
        } else {
            panic!()
        }
    }
}
```

### Give the rust wrapper a different name to the dll export
```rust
#[dll("ntdll.dll")]
extern "system" {
    #[link_name = "RtlGetVersion"]
    fn rtl_get_version(lp_version_information: *mut OSVERSIONINFOW) -> NTSTATUS;
}
```

### Use a dll export without a name
```rust
#[dll("uxtheme.dll")]
extern "system" {
    #[link_ordinal = 133]
    fn allow_dark_mode_for_window(hwnd: HWND, allow: BOOL) -> BOOL;
}
```

### Check whether a function exists
```rust
#[dll("ntdll.dll")]
extern "system" {
    #[link_name = "RtlGetVersion"]
    fn rtl_get_version(lp_version_information: *mut OSVERSIONINFOW) -> NTSTATUS;
}

fn rtl_get_version_exists() -> bool {
    rtl_get_version::exists()
}
```

### Pass flags to the underlying LoadLibraryExW call

```rust
use windows_dll::*;
#[dll("ntdll.dll", LOAD_LIBRARY_SEARCH_SYSTEM32)]
extern "system" {
    #[link_name = "RtlGetVersion"]
    fn rtl_get_version(lp_version_information: *mut OSVERSIONINFOW) -> NTSTATUS;
}

fn rtl_get_version_exists() -> bool {
    rtl_get_version::exists()
}
```