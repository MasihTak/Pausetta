use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::commands::AppSettingsService;
use crate::settings::model::PauseKind;
use crate::settings_window;

const SETTINGS_ID: &str = "settings";
const ABOUT_ID: &str = "about";
const PAUSE_ONE_HOUR_ID: &str = "pause-one-hour";
const PAUSE_TODAY_ID: &str = "pause-today";
const RESUME_ID: &str = "resume";
const QUIT_ID: &str = "quit";

/// Emitted after the tray changes settings, so an already-open Settings window re-reads
/// them instead of showing a stale pause state.
pub const SETTINGS_CHANGED_EVENT: &str = "settings-changed";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("Pausetta")
        .on_menu_event(handle_menu_event);

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let settings = MenuItem::with_id(app, SETTINGS_ID, "Settings", true, None::<&str>)?;
    let about = MenuItem::with_id(app, ABOUT_ID, "About", true, None::<&str>)?;

    // The three pause items stay enabled unconditionally: Resume with nothing paused is a
    // harmless no-op, and greying them out would mean tracking menu state across every change.
    let pause_one_hour =
        MenuItem::with_id(app, PAUSE_ONE_HOUR_ID, "Pause 1 hour", true, None::<&str>)?;
    let pause_today = MenuItem::with_id(app, PAUSE_TODAY_ID, "Pause today", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, RESUME_ID, "Resume", true, None::<&str>)?;
    let pause = Submenu::with_items(
        app,
        "Pause",
        true,
        &[&pause_one_hour, &pause_today, &resume],
    )?;

    let quit = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;

    Menu::with_items(
        app,
        &[
            &settings,
            &about,
            &PredefinedMenuItem::separator(app)?,
            &pause,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let to_message = |error: tauri::Error| error.to_string();
    let result = match event.id().as_ref() {
        SETTINGS_ID => settings_window::show(app).map_err(to_message),
        ABOUT_ID => settings_window::show_about(app).map_err(to_message),
        PAUSE_ONE_HOUR_ID => change_pause(app, Some(PauseKind::OneHour)),
        PAUSE_TODAY_ID => change_pause(app, Some(PauseKind::Today)),
        RESUME_ID => change_pause(app, None),
        QUIT_ID => {
            // The only real exit; every window close just hides, see lib.rs.
            app.exit(0);
            Ok(())
        }
        _ => Ok(()),
    };

    if let Err(error) = result {
        eprintln!("[pausetta] tray action '{}' failed: {error}", event.id().0);
    }
}

fn change_pause(app: &AppHandle, kind: Option<PauseKind>) -> Result<(), String> {
    let service = app.state::<AppSettingsService>();
    match kind {
        Some(kind) => service.pause(kind)?,
        None => service.resume()?,
    };
    app.emit(SETTINGS_CHANGED_EVENT, ())
        .map_err(|error| error.to_string())
}
