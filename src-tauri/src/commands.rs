use tauri::{AppHandle, State, WebviewWindow};

use crate::autostart::SystemLoginItem;
use crate::reminder_window;
use crate::scheduler::idle::SystemIdleDetector;
use crate::scheduler::service::SchedulerService;
use crate::settings::error::SettingsError;
use crate::settings::model::{CategoryKey, PauseKind, Settings};
use crate::settings::repository::SqliteSettingsRepository;
use crate::settings::service::SettingsService;

pub type AppSettingsService = SettingsService<SqliteSettingsRepository>;
pub type AppSchedulerService = SchedulerService<SystemIdleDetector>;

#[tauri::command]
pub fn get_settings(service: State<'_, AppSettingsService>) -> Settings {
    service.get()
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    service: State<'_, AppSettingsService>,
    settings: Settings,
) -> Result<Settings, SettingsError> {
    service.update(settings, &SystemLoginItem(&app))
}

#[tauri::command]
pub fn pause_reminders(
    service: State<'_, AppSettingsService>,
    kind: PauseKind,
) -> Result<Settings, SettingsError> {
    service.pause(kind)
}

#[tauri::command]
pub fn resume_reminders(service: State<'_, AppSettingsService>) -> Result<Settings, SettingsError> {
    service.resume()
}

// Window commands are async: creating a window from a sync command deadlocks on Windows.
#[tauri::command]
pub async fn show_reminder(app: AppHandle, category: CategoryKey) -> Result<(), String> {
    reminder_window::show(&app, category).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn close_reminder(window: WebviewWindow) -> Result<(), String> {
    reminder_window::close(&window).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn snooze_reminder(
    window: WebviewWindow,
    scheduler: State<'_, AppSchedulerService>,
    category: CategoryKey,
) -> Result<(), String> {
    scheduler.snooze(category);
    reminder_window::close(&window).map_err(|error| error.to_string())
}
