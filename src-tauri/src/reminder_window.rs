use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder,
};

use crate::settings::model::CategoryKey;

const REMINDER_WINDOW_LABEL_PREFIX: &str = "reminder-";
const REMINDER_CHANGED_EVENT: &str = "reminder-changed";
// Larger than the 352px toast card so its glow and shadow fit inside the transparent window.
const WINDOW_WIDTH: f64 = 408.0;
const WINDOW_HEIGHT: f64 = 320.0;

// The label of the toast that can still take a new reminder. `close` clears it before the
// window starts closing, because a closing window stays registered for a moment and a
// reminder sent to it would be lost. Each toast gets a fresh label for the same reason:
// the next one can open while the old one is still registered.
static CURRENT_LABEL: Mutex<Option<String>> = Mutex::new(None);
static NEXT_TOAST_ID: AtomicU64 = AtomicU64::new(0);

pub fn show(app: &AppHandle, category: CategoryKey) -> tauri::Result<()> {
    // Held until the window is built, so a concurrent `close` can't slip in between.
    let mut current_label = lock_current_label();
    let open_label = current_label
        .as_deref()
        .filter(|label| app.get_webview_window(label).is_some());
    if let Some(label) = open_label {
        return app.emit_to(label, REMINDER_CHANGED_EVENT, category);
    }

    // The frontend reads these globals to render the toast instead of the Settings UI.
    // The anchor mirrors corner_position()'s own platform check below, so there is one
    // source of truth for "which corner" instead of the frontend re-deriving it from
    // navigator.userAgent.
    let anchor = if cfg!(target_os = "macos") {
        "top"
    } else {
        "bottom"
    };
    let initialization_script = format!(
        "window.__PAUSETTA_REMINDER_CATEGORY__ = \"{}\"; window.__PAUSETTA_REMINDER_ANCHOR__ = \"{}\";",
        category.as_str(),
        anchor
    );

    let label = format!(
        "{REMINDER_WINDOW_LABEL_PREFIX}{}",
        NEXT_TOAST_ID.fetch_add(1, Ordering::Relaxed)
    );
    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title("Pausetta reminder")
        .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        // A reminder must never steal focus from whatever the user is typing into.
        .focused(false)
        .initialization_script(&initialization_script);

    if let Some(position) = corner_position(app)? {
        builder = builder.position(position.x, position.y);
    }

    builder.build()?;
    *current_label = Some(label);
    Ok(())
}

pub fn close(app: &AppHandle) -> tauri::Result<()> {
    let Some(label) = lock_current_label().take() else {
        return Ok(());
    };
    if let Some(window) = app.get_webview_window(&label) {
        window.close()?;
    }
    Ok(())
}

/// A poisoned lock only means a panic mid-show; the label it holds is still usable.
fn lock_current_label() -> MutexGuard<'static, Option<String>> {
    CURRENT_LABEL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Top-right on macOS (where system notifications appear), bottom-right elsewhere,
/// kept inside the work area so the menu bar or taskbar never covers it.
fn corner_position(app: &AppHandle) -> tauri::Result<Option<LogicalPosition<f64>>> {
    let Some(monitor) = app.primary_monitor()? else {
        return Ok(None);
    };
    let scale_factor = monitor.scale_factor();
    let area_position: LogicalPosition<f64> = monitor.work_area().position.to_logical(scale_factor);
    let area_size: LogicalSize<f64> = monitor.work_area().size.to_logical(scale_factor);

    let x = area_position.x + area_size.width - WINDOW_WIDTH;
    let y = if cfg!(target_os = "macos") {
        area_position.y
    } else {
        area_position.y + area_size.height - WINDOW_HEIGHT
    };
    Ok(Some(LogicalPosition::new(x, y)))
}
