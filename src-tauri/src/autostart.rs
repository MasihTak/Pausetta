use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Brings the OS login item in line with the persisted setting.
///
/// Called at startup as well as on change, so a login item removed outside the app — or
/// left pointing at an old install path — is repaired on the next launch.
pub fn apply(app: &AppHandle, launch_on_login: bool) -> Result<(), String> {
    // A dev run would register its own target/debug executable as the login item.
    if tauri::is_dev() {
        return Ok(());
    }

    let manager = app.autolaunch();
    if launch_on_login {
        // Always rewritten: is_enabled() only reports that an entry exists, not that it
        // points at this executable.
        return manager.enable().map_err(|error| error.to_string());
    }

    let is_enabled = manager.is_enabled().map_err(|error| error.to_string())?;
    if !is_enabled {
        return Ok(());
    }
    manager.disable().map_err(|error| error.to_string())
}

/// Whether the user switched Pausetta off in Task Manager's Startup apps.
///
/// Callers check this before `apply`, which would otherwise switch it back on.
#[cfg(windows)]
pub fn is_disabled_in_startup_apps(app: &AppHandle) -> bool {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    const STARTUP_APPROVED_KEY: &str =
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

    if tauri::is_dev() {
        return false;
    }

    // A missing key or value means Windows was never told to hold the entry back.
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(STARTUP_APPROVED_KEY, KEY_READ)
        .and_then(|key| key.get_raw_value(&app.package_info().name))
        .is_ok_and(|value| is_disabled_startup_value(&value.bytes))
}

#[cfg(not(windows))]
pub fn is_disabled_in_startup_apps(_app: &AppHandle) -> bool {
    false
}

/// Windows marks a disabled entry with an odd first byte (03, 07); enabled is 02 or 06.
#[cfg_attr(not(windows), allow(dead_code))]
fn is_disabled_startup_value(bytes: &[u8]) -> bool {
    bytes.first().is_some_and(|state| state & 1 == 1)
}

#[cfg(test)]
mod tests {
    use super::is_disabled_startup_value;

    #[test]
    fn reads_the_startup_apps_state_from_the_first_byte() {
        let disabled_in_task_manager = [3, 0, 0, 0, 0x5a, 0x11, 0x9c, 0x2e, 0x41, 0x18, 0xdc, 1];
        let enabled_by_auto_launch = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        assert!(is_disabled_startup_value(&disabled_in_task_manager));
        assert!(!is_disabled_startup_value(&enabled_by_auto_launch));
        assert!(!is_disabled_startup_value(&[]));
    }
}
