use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use serde::Deserialize;
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow,
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
const CURSOR_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The toast card inside its window, in CSS pixels from the window's top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct CardBounds {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

struct Toast {
    label: String,
    /// False between reserving the slot and the window existing, so a second toast can't
    /// mistake a window that is still being built for one the OS closed.
    is_built: bool,
    card: Option<CardBounds>,
}

#[derive(Default)]
struct StackState {
    /// The toast in each stack slot. `close` frees a slot before its window starts
    /// closing, because a closing window stays registered for a moment.
    slots: Vec<Option<Toast>>,
    is_polling_cursor: bool,
}

impl StackState {
    fn find_mut(&mut self, label: &str) -> Option<&mut Toast> {
        self.slots
            .iter_mut()
            .flatten()
            .find(|toast| toast.label == label)
    }
}

/// Which toasts are on screen and where they stack. Each toast gets a fresh label, because
/// the next one can open while the old one is still registered.
#[derive(Default)]
pub struct ToastStack {
    state: Mutex<StackState>,
    next_toast_id: AtomicU64,
}

impl ToastStack {
    fn reserve_slot(&self, is_window_open: impl Fn(&str) -> bool) -> (usize, String) {
        let id = self.next_toast_id.fetch_add(1, Ordering::Relaxed);
        let label = format!("{REMINDER_WINDOW_LABEL_PREFIX}{id}");
        let mut state = self.lock();
        let slot = find_free_slot(&state.slots, is_window_open);
        let toast = Some(Toast {
            label: label.clone(),
            is_built: false,
            card: None,
        });
        match state.slots.get_mut(slot) {
            Some(reserved_slot) => *reserved_slot = toast,
            None => state.slots.push(toast),
        }
        (slot, label)
    }

    /// Returns whether the caller must start the cursor poll: it runs only while a toast does.
    fn mark_built(&self, label: &str) -> bool {
        let mut state = self.lock();
        if let Some(toast) = state.find_mut(label) {
            toast.is_built = true;
        }
        !std::mem::replace(&mut state.is_polling_cursor, true)
    }

    fn release(&self, label: &str) {
        self.lock()
            .slots
            .iter_mut()
            .filter(|slot| slot.as_ref().is_some_and(|toast| toast.label == label))
            .for_each(|slot| *slot = None);
    }

    pub fn set_card(&self, label: &str, card: CardBounds) {
        if let Some(toast) = self.lock().find_mut(label) {
            toast.card = Some(card);
        }
    }

    /// The cards to track right now, or `None` once no toast is left, which also ends the poll.
    fn cards_to_track(&self) -> Option<Vec<(String, CardBounds)>> {
        let mut state = self.lock();
        let built_toasts: Vec<(String, Option<CardBounds>)> = state
            .slots
            .iter()
            .flatten()
            .filter(|toast| toast.is_built)
            .map(|toast| (toast.label.clone(), toast.card))
            .collect();
        if built_toasts.is_empty() {
            state.is_polling_cursor = false;
            return None;
        }
        Some(
            built_toasts
                .into_iter()
                .filter_map(|(label, card)| Some((label, card?)))
                .collect(),
        )
    }

    /// A poisoned lock only means a panic mid-update; the slots it holds are still usable.
    fn lock(&self) -> MutexGuard<'_, StackState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The first slot that is empty or whose window the OS closed behind our back, else a new one.
fn find_free_slot(slots: &[Option<Toast>], is_window_open: impl Fn(&str) -> bool) -> usize {
    slots
        .iter()
        .position(|slot| match slot {
            None => true,
            Some(toast) => toast.is_built && !is_window_open(&toast.label),
        })
        .unwrap_or(slots.len())
}

/// Every reminder gets its own window, so one coming due while another is still on screen
/// stacks next to it instead of replacing it.
pub fn show(app: &AppHandle, category: CategoryKey) -> tauri::Result<()> {
    let stack = app.state::<ToastStack>();
    // Reserved before building, with the lock released, so a slow window build never blocks
    // other toasts closing and two reminders can't claim the same slot.
    let (slot, label) = stack.reserve_slot(|label| app.get_webview_window(label).is_some());

    if let Err(error) = build_window(app, category, slot, &label) {
        stack.release(&label);
        return Err(error);
    }
    if stack.mark_built(&label) {
        spawn_cursor_poll(app);
    }
    Ok(())
}

fn build_window(
    app: &AppHandle,
    category: CategoryKey,
    slot: usize,
    label: &str,
) -> tauri::Result<()> {
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

    let mut builder = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
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
    Ok(())
}

/// Closes one toast, leaving any others stacked with it on screen.
pub fn close(window: &WebviewWindow) -> tauri::Result<()> {
    window
        .app_handle()
        .state::<ToastStack>()
        .release(window.label());
    window.close()
}

/// The window is larger than the card to fit its glow. That invisible margin must not
/// swallow clicks meant for the apps underneath, and a click-through window receives no
/// mouse events, so one task polls the cursor for every toast and switches click-through
/// off while the cursor is over a card.
fn spawn_cursor_poll(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut last_applied = HashMap::new();
        let mut has_logged_failure = false;
        loop {
            tokio::time::sleep(CURSOR_POLL_INTERVAL).await;
            let Some(cards) = app.state::<ToastStack>().cards_to_track() else {
                return;
            };
            match apply_click_through(&app, &cards, &mut last_applied) {
                Ok(()) => has_logged_failure = false,
                // Once per outage: at 20 polls a second a persistent failure would flood the log.
                Err(error) if !has_logged_failure => {
                    log::error!("could not update toast click-through: {error}");
                    has_logged_failure = true;
                }
                Err(_) => {}
            }
        }
    });
}

fn apply_click_through(
    app: &AppHandle,
    cards: &[(String, CardBounds)],
    last_applied: &mut HashMap<String, bool>,
) -> tauri::Result<()> {
    let cursor = app.cursor_position()?;
    for (label, card) in cards {
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        let should_ignore_cursor = !is_cursor_over_card(
            cursor,
            window.inner_position()?,
            window.scale_factor()?,
            card,
        );
        // Only on change: each call is a round trip to the OS.
        if last_applied.get(label) != Some(&should_ignore_cursor) {
            window.set_ignore_cursor_events(should_ignore_cursor)?;
            last_applied.insert(label.clone(), should_ignore_cursor);
        }
    }
    Ok(())
}

fn is_cursor_over_card(
    cursor: PhysicalPosition<f64>,
    window_origin: PhysicalPosition<i32>,
    scale_factor: f64,
    card: &CardBounds,
) -> bool {
    let x = (cursor.x - f64::from(window_origin.x)) / scale_factor;
    let y = (cursor.y - f64::from(window_origin.y)) / scale_factor;
    x >= card.left && x < card.right && y >= card.top && y < card.bottom
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

    fn built_toast(label: &str) -> Option<Toast> {
        Some(Toast {
            label: label.into(),
            is_built: true,
            card: None,
        })
    }

    fn card() -> CardBounds {
        CardBounds {
            left: 28.0,
            top: 100.0,
            right: 380.0,
            bottom: 280.0,
        }
    }

    #[test]
    fn a_new_toast_takes_the_first_empty_slot() {
        let slots = [built_toast("reminder-0"), None, built_toast("reminder-2")];

        assert_eq!(find_free_slot(&slots, |_| true), 1);
    }

    #[test]
    fn a_new_toast_stacks_after_the_open_ones() {
        let slots = [built_toast("reminder-0"), built_toast("reminder-1")];

        assert_eq!(find_free_slot(&slots, |_| true), 2);
    }

    #[test]
    fn a_slot_whose_window_the_os_closed_is_reused() {
        let slots = [built_toast("reminder-0"), built_toast("reminder-1")];

        assert_eq!(find_free_slot(&slots, |label| label != "reminder-0"), 0);
    }

    #[test]
    fn a_window_still_being_built_keeps_its_slot() {
        let pending = Some(Toast {
            label: "reminder-0".into(),
            is_built: false,
            card: None,
        });

        assert_eq!(find_free_slot(&[pending], |_| false), 1);
    }

    #[test]
    fn the_stack_hands_out_distinct_slots_and_reuses_released_ones() {
        let stack = ToastStack::default();

        let (first_slot, first_label) = stack.reserve_slot(|_| true);
        let (second_slot, _) = stack.reserve_slot(|_| true);
        assert_eq!((first_slot, second_slot), (0, 1));

        stack.release(&first_label);
        let (reused_slot, _) = stack.reserve_slot(|_| true);
        assert_eq!(reused_slot, 0);
    }

    #[test]
    fn the_cursor_poll_runs_only_while_a_toast_is_built() {
        let stack = ToastStack::default();
        let (_, label) = stack.reserve_slot(|_| true);

        assert!(stack.mark_built(&label), "the first toast starts the poll");
        assert!(!stack.mark_built(&label), "a running poll starts only once");
        assert_eq!(stack.cards_to_track(), Some(vec![]), "no card reported yet");

        stack.set_card(&label, card());
        assert_eq!(stack.cards_to_track(), Some(vec![(label.clone(), card())]));

        stack.release(&label);
        assert_eq!(stack.cards_to_track(), None);
        assert!(stack.mark_built(&label), "a new toast restarts the poll");
    }

    #[test]
    fn clicks_land_on_the_card_only_inside_its_bounds() {
        // A 2x screen with the window at (1000, 500).
        let origin = PhysicalPosition::new(1000, 500);
        let is_over_card =
            |x, y| is_cursor_over_card(PhysicalPosition::new(x, y), origin, 2.0, &card());

        // (1010, 510) physical is (5, 5) in the window: the empty margin.
        assert!(!is_over_card(1010.0, 510.0));
        // (1400, 900) physical is (200, 200) in the window: on the card.
        assert!(is_over_card(1400.0, 900.0));
        // The left and top edges are inside the card, the right edge is outside.
        assert!(is_over_card(1000.0 + 28.0 * 2.0, 900.0));
        assert!(!is_over_card(1000.0 + 380.0 * 2.0, 900.0));
    }
}
