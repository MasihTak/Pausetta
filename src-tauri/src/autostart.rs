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
