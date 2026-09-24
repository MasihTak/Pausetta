use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

pub const SETTINGS_WINDOW_LABEL: &str = "settings";
const SHOW_SCREEN_EVENT: &str = "show-screen";

const WINDOW_WIDTH: f64 = 460.0;
const WINDOW_HEIGHT: f64 = 760.0;
const MIN_WINDOW_HEIGHT: f64 = 360.0;
// inner_size excludes the title bar.
const TITLE_BAR_ALLOWANCE: f64 = 40.0;

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    show_screen(app, "settings")
}

pub fn show_about(app: &AppHandle) -> tauri::Result<()> {
    show_screen(app, "about")
}

/// Opens the window on a given screen, creating it on first use.
///
/// An existing window is told which screen to show with an event; a brand new one is told
/// with an injected global, because the webview has no listener registered yet when it is
/// created. That mirrors how `reminder_window` hands the toast its category.
fn show_screen(app: &AppHandle, screen: &str) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
        return app.emit_to(SETTINGS_WINDOW_LABEL, SHOW_SCREEN_EVENT, screen);
    }

    let initialization_script = format!("window.__PAUSETTA_SCREEN__ = \"{screen}\";");
    let window = WebviewWindowBuilder::new(
        app,
        SETTINGS_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("Pausetta")
    .inner_size(WINDOW_WIDTH, fitted_height(app)?)
    .center()
    .resizable(false)
    // The app lives in the tray, so none of its windows belong in the taskbar.
    .skip_taskbar(true)
    .initialization_script(&initialization_script)
    .build()?;

    // Hide rather than destroy, so reopening from the tray is instant and the webview
    // keeps the settings it has already loaded.
    let window_to_hide = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window_to_hide.hide();
        }
    });

    Ok(())
}

/// The window isn't resizable, so it must fit the screen; the page scrolls if needed.
fn fitted_height(app: &AppHandle) -> tauri::Result<f64> {
    let Some(monitor) = app.primary_monitor()? else {
        return Ok(WINDOW_HEIGHT);
    };
    let work_area: LogicalSize<f64> = monitor.work_area().size.to_logical(monitor.scale_factor());
    Ok(WINDOW_HEIGHT
        .min(work_area.height - TITLE_BAR_ALLOWANCE)
        .max(MIN_WINDOW_HEIGHT))
}
