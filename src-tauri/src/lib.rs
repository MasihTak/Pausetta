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
                eprintln!("[pausetta] could not open settings for a second launch: {error}");
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ));

    builder
        .setup(|app| {
            // No dock icon: this is a tray app, not something you alt-tab to.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let repository = SqliteSettingsRepository::open(&data_dir.join("pausetta.db"))?;
            app.manage(SettingsService::new(repository));
            app.manage(SchedulerService::new(SystemIdleDetector));

            let handle = app.handle();
            let settings = app.state::<AppSettingsService>().get()?;
            // Non-fatal: a broken login item must not stop the reminders.
            if let Err(error) = autostart::apply(handle, settings.launch_on_login) {
                eprintln!("[pausetta] could not sync the login item: {error}");
            }
            tray::create(handle)?;
            scheduler::spawn(handle);

            // The app otherwise boots to the tray with nothing on screen. On a first run
            // there is no reason to expect an icon down there, so the walkthrough — which
            // is what points at the tray — opens itself once.
            if !settings.onboarding_completed {
                settings_window::show(handle)?;
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
