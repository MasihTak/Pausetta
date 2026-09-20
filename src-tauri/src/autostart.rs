use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Brings the OS login item in line with the persisted setting.
///
/// Called at startup as well as on change, so a login item removed outside the app — or
/// left pointing at an old install path — is repaired on the next launch.
pub fn apply(app: &AppHandle, launch_on_login: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    let is_enabled = manager.is_enabled().map_err(|error| error.to_string())?;
    if is_enabled == launch_on_login {
        return Ok(());
    }

    let change = if launch_on_login {
        manager.enable()
    } else {
        manager.disable()
    };
    change.map_err(|error| error.to_string())
}
