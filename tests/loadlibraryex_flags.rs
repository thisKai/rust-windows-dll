use windows_dll::{dll, flags::*};

use winapi::shared::{
    minwindef::{BOOL, ULONG},
    ntdef::LPCWSTR,
};

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
        bcrypt_add_context_function::exists(),
        "Didn't find bcrypt.dll in system dir..."
    );
    assert!(
        !fw_add_firewall_rule::exists(),
        "Found firewallapi.dll in application dir..."
    );
}
