use {
    minifb::{Key, Window, WindowOptions},
    once_cell::sync::Lazy,
    std::mem,
    winapi::shared::{
        basetsd::SIZE_T,
        minwindef::{BOOL, DWORD, FALSE, ULONG},
        ntdef::{NTSTATUS, NT_SUCCESS, PVOID, WCHAR},
        windef::HWND,
    },
    windows_dll::dll,
};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    dark_dwm_decorations(window.get_window_handle() as _, true);

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

static WIN10_BUILD: Lazy<Option<DWORD>> = Lazy::new(|| {
    #[dll(ntdll)]
    extern "system" {
        #[allow(non_snake_case)]
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
    if !RtlGetVersion::exists() {
        return None;
    }
    unsafe {
        let mut version_info = OSVERSIONINFOW {
            dwOSVersionInfoSize: 0,
            dwMajorVersion: 0,
            dwMinorVersion: 0,
            dwBuildNumber: 0,
            dwPlatformId: 0,
            szCSDVersion: [0; 128],
        };
        let status = RtlGetVersion(&mut version_info);

        if NT_SUCCESS(status)
            && version_info.dwMajorVersion == 10
            && version_info.dwMinorVersion == 0
        {
            Some(version_info.dwBuildNumber)
        } else {
            None
        }
    }
});

static DARK_MODE_SUPPORTED: Lazy<bool> = Lazy::new(|| match *WIN10_BUILD {
    Some(build) => build >= 17763,
    None => false,
});

fn dark_dwm_decorations(hwnd: HWND, enable_dark_mode: bool) -> bool {
    #[allow(non_snake_case)]
    type WINDOWCOMPOSITIONATTRIB = u32;
    const WCA_USEDARKMODECOLORS: WINDOWCOMPOSITIONATTRIB = 26;

    #[allow(non_snake_case)]
    #[repr(C)]
    struct WINDOWCOMPOSITIONATTRIBDATA {
        Attrib: WINDOWCOMPOSITIONATTRIB,
        pvData: PVOID,
        cbData: SIZE_T,
    }

    #[dll(user32)]
    extern "system" {
        #[allow(non_snake_case)]
        fn SetWindowCompositionAttribute(
            h_wnd: HWND,
            data: *mut WINDOWCOMPOSITIONATTRIBDATA,
        ) -> BOOL;
    }

    if *DARK_MODE_SUPPORTED && SetWindowCompositionAttribute::exists() {
        unsafe {
            let mut is_dark_mode_bigbool = enable_dark_mode as BOOL;
            let mut data = WINDOWCOMPOSITIONATTRIBDATA {
                Attrib: WCA_USEDARKMODECOLORS,
                pvData: &mut is_dark_mode_bigbool as *mut _ as _,
                cbData: mem::size_of::<BOOL>(),
            };

            let status = SetWindowCompositionAttribute(hwnd, &mut data);

            status != FALSE
        }
    } else {
        false
    }
}
