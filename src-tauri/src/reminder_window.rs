use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

use crate::settings::model::CategoryKey;

const REMINDER_WINDOW_LABEL_PREFIX: &str = "reminder-";
// Larger than the 352px toast card so its glow and shadow fit inside the transparent window.
const WINDOW_WIDTH: f64 = 408.0;
const WINDOW_HEIGHT: f64 = 320.0;
// The distance between stacked toasts: the 352x215 card plus a gap. Smaller than the window,
// so it is the windows' transparent margins that overlap, never the cards.
const STACK_OFFSET_X: f64 = 364.0;
const STACK_OFFSET_Y: f64 = 232.0;

// The label of the toast in each stack slot. `close` frees a slot before its window starts
// closing, because a closing window stays registered for a moment. Each toast gets a fresh
// label for the same reason: the next one can open while the old one is still registered.
static STACK_SLOTS: Mutex<Vec<Option<String>>> = Mutex::new(Vec::new());
static NEXT_TOAST_ID: AtomicU64 = AtomicU64::new(0);

/// Every reminder gets its own window, so one coming due while another is still on screen
/// stacks next to it instead of replacing it.
pub fn show(app: &AppHandle, category: CategoryKey) -> tauri::Result<()> {
    // Held until the window is built, so two reminders can't claim the same slot.
    let mut slots = lock_stack_slots();
    // The window check catches a toast the OS closed without going through `close`.
    let slot = slots
        .iter()
        .position(|label| {
            let is_open = label
                .as_deref()
                .is_some_and(|label| app.get_webview_window(label).is_some());
            !is_open
        })
        .unwrap_or(slots.len());

    // The frontend reads these globals to render the toast instead of the Settings UI.
    // The anchor mirrors slot_position()'s own platform check below, so there is one
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

    if let Some(position) = slot_position(app, slot)? {
        builder = builder.position(position.x, position.y);
    }

    builder.build()?;
    match slots.get_mut(slot) {
        Some(freed_slot) => *freed_slot = Some(label),
        None => slots.push(Some(label)),
    }
    Ok(())
}

/// Closes one toast, leaving any others stacked with it on screen.
pub fn close(window: &WebviewWindow) -> tauri::Result<()> {
    lock_stack_slots()
        .iter_mut()
        .filter(|label| label.as_deref() == Some(window.label()))
        .for_each(|label| *label = None);
    window.close()
}

/// A poisoned lock only means a panic mid-show; the slots it holds are still usable.
fn lock_stack_slots() -> MutexGuard<'static, Vec<Option<String>>> {
    STACK_SLOTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// How far a slot sits from the corner, as (columns across, rows along the edge). A new
/// column starts once the work area is too short for another row, so a toast is never
/// pushed off screen or onto another one.
fn stack_offset(slot: usize, area_height: f64) -> (f64, f64) {
    let spare_height = (area_height - WINDOW_HEIGHT).max(0.0);
    let rows_per_column = (spare_height / STACK_OFFSET_Y) as usize + 1;
    (
        (slot / rows_per_column) as f64 * STACK_OFFSET_X,
        (slot % rows_per_column) as f64 * STACK_OFFSET_Y,
    )
}

/// Top-right on macOS (where system notifications appear), bottom-right elsewhere,
/// kept inside the work area so the menu bar or taskbar never covers it.
fn slot_position(app: &AppHandle, slot: usize) -> tauri::Result<Option<LogicalPosition<f64>>> {
    let Some(monitor) = app.primary_monitor()? else {
        return Ok(None);
    };
    let scale_factor = monitor.scale_factor();
    let area_position: LogicalPosition<f64> = monitor.work_area().position.to_logical(scale_factor);
    let area_size: LogicalSize<f64> = monitor.work_area().size.to_logical(scale_factor);
    let (offset_x, offset_y) = stack_offset(slot, area_size.height);

    let x = area_position.x + area_size.width - WINDOW_WIDTH - offset_x;
    let y = if cfg!(target_os = "macos") {
        area_position.y + offset_y
    } else {
        area_position.y + area_size.height - WINDOW_HEIGHT - offset_y
    };
    Ok(Some(LogicalPosition::new(x, y)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toasts_stack_in_a_column_while_the_screen_has_room() {
        let tall_screen = 1040.0;

        assert_eq!(stack_offset(0, tall_screen), (0.0, 0.0));
        assert_eq!(stack_offset(1, tall_screen), (0.0, STACK_OFFSET_Y));
        assert_eq!(stack_offset(3, tall_screen), (0.0, 3.0 * STACK_OFFSET_Y));
    }

    #[test]
    fn a_short_screen_starts_a_second_column_instead_of_running_off_the_top() {
        // 1080p at Windows' default 150% scaling leaves room for two rows.
        let short_screen = 680.0;

        assert_eq!(stack_offset(1, short_screen), (0.0, STACK_OFFSET_Y));
        assert_eq!(stack_offset(2, short_screen), (STACK_OFFSET_X, 0.0));
        assert_eq!(
            stack_offset(3, short_screen),
            (STACK_OFFSET_X, STACK_OFFSET_Y)
        );
    }

    #[test]
    fn a_screen_shorter_than_one_toast_still_gets_a_row() {
        assert_eq!(stack_offset(1, 200.0), (STACK_OFFSET_X, 0.0));
    }
}
