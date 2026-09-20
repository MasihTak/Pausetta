pub mod idle;
pub mod service;

use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::commands::{AppSchedulerService, AppSettingsService};
use crate::reminder_window;
use crate::sound;

const TICK_INTERVAL: Duration = Duration::from_secs(1);

/// Starts the one task that decides when reminders fire. It runs off the main thread,
/// which is also what lets it open the reminder window without deadlocking on Windows.
pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(TICK_INTERVAL).await;
            tick(&app);
        }
    });
}

fn tick(app: &AppHandle) {
    // Re-read settings every tick: microseconds against an in-process database, and it keeps
    // the scheduler honest about edits made in the UI and about a pause that has just expired.
    let settings = match app.state::<AppSettingsService>().get() {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("[pausetta] could not read settings: {error}");
            return;
        }
    };

    let Some(category) = app.state::<AppSchedulerService>().tick(&settings) else {
        return;
    };

    if settings.sound_enabled {
        sound::play_notification();
    }
    if let Err(error) = reminder_window::show(app, category) {
        eprintln!(
            "[pausetta] could not show the {} reminder: {error}",
            category.as_str()
        );
    }
}
