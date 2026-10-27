mod autostart;
mod commands;
mod reminder_window;
mod scheduler;
mod settings;
mod settings_window;
mod sound;
mod tray;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use autostart::SystemLoginItem;
use commands::AppSettingsService;
use scheduler::idle::SystemIdleDetector;
use scheduler::service::SchedulerService;
use settings::repository::SqliteSettingsRepository;
use settings::service::SettingsService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        // Must be registered first. A second launch opens Settings in the running copy
        // instead of starting a second scheduler.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Err(error) = settings_window::show(app) {
                log::error!("could not open settings for a second launch: {error}");
            }
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ));

    builder
        .setup(|app| {
            // The app has no window at launch, so a failure here would otherwise look like
            // it never started. Show it, then exit.
            if let Err(error) = initialize(app) {
                log::error!("startup failed: {error}");
                let handle = app.handle().clone();
                let log_dir = app
                    .path()
                    .app_log_dir()
                    .map(|dir| dir.display().to_string())
                    .unwrap_or_default();
                app.dialog()
                    .message(format!(
                        "Pausetta could not start: {error}\n\nDetails are in {log_dir}"
                    ))
                    .title("Pausetta")
                    .kind(MessageDialogKind::Error)
                    .show(move |_| handle.exit(1));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::update_settings,
            commands::pause_reminders,
            commands::resume_reminders,
            commands::show_reminder,
            commands::close_reminder,
            commands::snooze_reminder,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Closing the last window must not quit a tray app. A user-driven exit carries
            // no code; the tray's Quit calls exit(0) and so is allowed through.
            if let tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } = event
            {
                api.prevent_exit();
            }
        });
}

fn initialize(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // No dock icon: this is a tray app, not something you alt-tab to.
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    let repository = SqliteSettingsRepository::open(&data_dir.join("pausetta.db"))?;
    app.manage(SettingsService::new(repository)?);
    app.manage(SchedulerService::new(SystemIdleDetector));

    let handle = app.handle();
    let settings = app
        .state::<AppSettingsService>()
        .sync_login_item(&SystemLoginItem(handle))?;
    tray::create(handle)?;
    scheduler::spawn(handle);

    // The app otherwise boots to the tray with nothing on screen. On a first run
    // there is no reason to expect an icon down there, so the walkthrough — which
    // is what points at the tray — opens itself once.
    if !settings.onboarding_completed {
        settings_window::show(handle)?;
    }
    Ok(())
}
