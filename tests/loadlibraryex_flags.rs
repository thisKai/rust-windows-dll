use platform::*;
use windows_dll::{dll, flags::*};

#[dll("bcrypt", LOAD_LIBRARY_SEARCH_SYSTEM32)]
extern "system" {
    #[link_name = "BCryptAddContextFunction"]
    fn bcrypt_add_context_function(
        dw_table: ULONG,
        psz_context: LPCWSTR,
        dw_interface: ULONG,
        psz_function: LPCWSTR,
        dw_position: ULONG,
    ) -> BOOL;
}

#[dll("firewallapi.dll", LOAD_LIBRARY_SEARCH_APPLICATION_DIR)]
extern "system" {
    #[link_name = "FWAddFirewallRule"]
    pub fn fw_add_firewall_rule() -> ();
}

#[test]
fn assert_args_passed() {
    assert!(
        unsafe { bcrypt_add_context_function::exists() },
        "Didn't find bcrypt.dll in system dir..."
    );
    assert!(
        unsafe { !fw_add_firewall_rule::exists() },
        "Found firewallapi.dll in application dir..."
    );
}

#[cfg(feature = "winapi")]
mod platform {
    pub use winapi::shared::{
        minwindef::{BOOL, ULONG},
        ntdef::LPCWSTR,
    };
}

#[cfg(feature = "windows")]
mod platform {
    pub use windows::{core::PCWSTR as LPCWSTR, Win32::Foundation::BOOL};

    pub type ULONG = u32;
}

#[cfg(feature = "windows-sys")]
mod platform {
    pub use windows_sys::{core::PCWSTR as LPCWSTR, Win32::Foundation::BOOL};

    pub type ULONG = u32;
}
