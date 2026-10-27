use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Local, TimeDelta, TimeZone, Utc};

use crate::autostart::LoginItem;

use super::model::{PauseKind, PauseState, Settings};
use super::repository::SettingsRepository;

pub struct SettingsService<R: SettingsRepository> {
    repository: R,
    /// Serializes load → change → save, so concurrent writers can't revert each other.
    write_lock: Mutex<()>,
}

impl<R: SettingsRepository> SettingsService<R> {
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            write_lock: Mutex::new(()),
        }
    }

    pub fn get(&self) -> Result<Settings, String> {
        self.get_at(Utc::now())
    }

    pub fn update(
        &self,
        requested: Settings,
        login_item: &impl LoginItem,
    ) -> Result<Settings, String> {
        requested.validate()?;
        {
            let _write_guard = self.lock_writes()?;
            let current = self.repository.load()?;
            let mut settings = requested;
            // Switching the toggle on is a fresh request; an unrelated save must not undo a
            // Startup apps "Disabled".
            if is_held_back_by_user(&current, login_item) {
                settings.launch_on_login = false;
            }
            // OS first: if it refuses the login item, nothing is saved.
            login_item.apply(settings.launch_on_login)?;
            // Pause only changes through pause/resume, so a stale settings form can't undo it.
            settings.pause = current.pause.clone();
            if let Err(save_error) = self.repository.save(&settings) {
                if let Err(restore_error) = login_item.apply(current.launch_on_login) {
                    log::error!("could not restore the login item: {restore_error}");
                }
                return Err(save_error);
            }
        }
        self.get()
    }

    /// Startup counterpart of `update`: honours a Startup apps "Disabled", then repairs the
    /// OS login item so it matches the saved setting.
    pub fn sync_login_item(&self, login_item: &impl LoginItem) -> Result<Settings, String> {
        let mut settings = self.get()?;
        if is_held_back_by_user(&settings, login_item) {
            settings = self.modify(|settings| settings.launch_on_login = false)?;
        }
        // Non-fatal: a broken login item must not stop the reminders.
        if let Err(error) = login_item.apply(settings.launch_on_login) {
            log::error!("could not sync the login item: {error}");
        }
        Ok(settings)
    }

    pub fn pause(&self, kind: PauseKind) -> Result<Settings, String> {
        self.pause_at(kind, Utc::now())
    }

    pub fn resume(&self) -> Result<Settings, String> {
        self.modify(|settings| settings.pause = None)
    }

    fn get_at(&self, now: DateTime<Utc>) -> Result<Settings, String> {
        let settings = self.repository.load()?;
        if !has_expired_pause(&settings, now) {
            return Ok(settings);
        }
        // Re-checked under the lock: a new pause may have landed since.
        self.modify(|settings| {
            if has_expired_pause(settings, now) {
                settings.pause = None;
            }
        })
    }

    fn pause_at(&self, kind: PauseKind, now: DateTime<Utc>) -> Result<Settings, String> {
        let until = match kind {
            PauseKind::OneHour => now + TimeDelta::hours(1),
            PauseKind::Today => next_local_midnight(now),
        };
        self.modify(|settings| settings.pause = Some(PauseState { kind, until }))
    }

    fn modify(&self, change: impl FnOnce(&mut Settings)) -> Result<Settings, String> {
        let _write_guard = self.lock_writes()?;
        let mut settings = self.repository.load()?;
        change(&mut settings);
        self.repository.save(&settings)?;
        Ok(settings)
    }

    fn lock_writes(&self) -> Result<MutexGuard<'_, ()>, String> {
        self.write_lock
            .lock()
            .map_err(|_| "settings write lock was poisoned".to_string())
    }
}

fn is_held_back_by_user(current: &Settings, login_item: &impl LoginItem) -> bool {
    current.launch_on_login && login_item.is_disabled_by_user()
}

fn has_expired_pause(settings: &Settings, now: DateTime<Utc>) -> bool {
    settings
        .pause
        .as_ref()
        .is_some_and(|pause| pause.until <= now)
}

fn next_local_midnight(now: DateTime<Utc>) -> DateTime<Utc> {
    let tomorrow = now
        .with_timezone(&Local)
        .date_naive()
        .succ_opt()
        .expect("date overflow is unreachable for real clocks");
    let midnight = tomorrow
        .and_hms_opt(0, 0, 0)
        .expect("00:00:00 is a valid time");
    // Some time zones skip midnight on DST change days; fall back to a full day.
    Local
        .from_local_datetime(&midnight)
        .earliest()
        .map(|local_midnight| local_midnight.with_timezone(&Utc))
        .unwrap_or(now + TimeDelta::days(1))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use chrono::NaiveTime;

    use super::*;
    use crate::settings::repository::SqliteSettingsRepository;

    #[derive(Default)]
    struct FakeLoginItem {
        disabled_by_user: bool,
        refuses: bool,
        applied: RefCell<Vec<bool>>,
    }

    impl LoginItem for FakeLoginItem {
        fn is_disabled_by_user(&self) -> bool {
            self.disabled_by_user
        }

        fn apply(&self, launch_on_login: bool) -> Result<(), String> {
            if self.refuses {
                return Err("refused".into());
            }
            self.applied.borrow_mut().push(launch_on_login);
            Ok(())
        }
    }

    struct FailingSaveRepository(Settings);

    impl SettingsRepository for FailingSaveRepository {
        fn load(&self) -> Result<Settings, String> {
            Ok(self.0.clone())
        }

        fn save(&self, _settings: &Settings) -> Result<(), String> {
            Err("disk full".into())
        }
    }

    fn launching_on_login() -> Settings {
        Settings {
            launch_on_login: true,
            ..Settings::default()
        }
    }

    fn disabled_in_startup_apps() -> FakeLoginItem {
        FakeLoginItem {
            disabled_by_user: true,
            ..FakeLoginItem::default()
        }
    }

    fn service() -> SettingsService<SqliteSettingsRepository> {
        SettingsService::new(SqliteSettingsRepository::open_in_memory().unwrap())
    }

    #[test]
    fn seeds_defaults_on_first_open() {
        assert_eq!(service().get().unwrap(), Settings::default());
    }

    #[test]
    fn update_round_trips() {
        let service = service();
        let mut requested = Settings::default();
        requested.categories.eye.interval_minutes = 45;
        requested.categories.hydration.enabled = false;
        requested.quiet_hours.start = "21:30".into();
        requested.sound_enabled = true;
        requested.onboarding_completed = true;

        service
            .update(requested.clone(), &FakeLoginItem::default())
            .unwrap();

        assert_eq!(service.get().unwrap(), requested);
    }

    #[test]
    fn rejects_invalid_values_without_saving() {
        let service = service();
        let mut invalid_interval = Settings::default();
        invalid_interval.categories.posture.interval_minutes = 7;
        let mut invalid_time = Settings::default();
        invalid_time.quiet_hours.end = "08:15".into();

        assert!(service
            .update(invalid_interval, &FakeLoginItem::default())
            .is_err());
        assert!(service
            .update(invalid_time, &FakeLoginItem::default())
            .is_err());
        assert_eq!(service.get().unwrap(), Settings::default());
    }

    #[test]
    fn pause_today_ends_at_next_local_midnight() {
        let now = Utc::now();
        let settings = service().pause_at(PauseKind::Today, now).unwrap();
        let until = settings.pause.unwrap().until;

        assert!(until > now);
        assert!(until - now <= TimeDelta::days(1));
        // On DST days where midnight doesn't exist, next_local_midnight falls back to a full day.
        let is_local_midnight = until.with_timezone(&Local).time() == NaiveTime::MIN;
        assert!(is_local_midnight || until == now + TimeDelta::days(1));
    }

    #[test]
    fn expired_pause_is_cleared_on_read() {
        let service = service();
        let now = Utc::now();
        service
            .pause_at(PauseKind::OneHour, now - TimeDelta::hours(2))
            .unwrap();

        assert_eq!(service.get_at(now).unwrap().pause, None);
    }

    #[test]
    fn update_does_not_clear_an_active_pause() {
        let service = service();
        service.pause(PauseKind::OneHour).unwrap();

        let updated = service
            .update(Settings::default(), &FakeLoginItem::default())
            .unwrap();

        assert_eq!(
            updated.pause.map(|pause| pause.kind),
            Some(PauseKind::OneHour)
        );
    }

    #[test]
    fn update_keeps_a_login_item_the_user_disabled_in_startup_apps_off() {
        let service = service();
        service
            .update(launching_on_login(), &FakeLoginItem::default())
            .unwrap();
        let login_item = disabled_in_startup_apps();

        let saved = service.update(launching_on_login(), &login_item).unwrap();

        assert!(!saved.launch_on_login);
        assert_eq!(*login_item.applied.borrow(), [false]);
    }

    #[test]
    fn update_treats_switching_launch_on_login_on_as_a_fresh_request() {
        let login_item = disabled_in_startup_apps();

        let saved = service().update(launching_on_login(), &login_item).unwrap();

        assert!(saved.launch_on_login);
        assert_eq!(*login_item.applied.borrow(), [true]);
    }

    #[test]
    fn update_saves_nothing_when_the_os_refuses_the_login_item() {
        let service = service();
        let login_item = FakeLoginItem {
            refuses: true,
            ..FakeLoginItem::default()
        };

        assert!(service.update(launching_on_login(), &login_item).is_err());
        assert_eq!(service.get().unwrap(), Settings::default());
    }

    #[test]
    fn update_restores_the_login_item_when_saving_fails() {
        let service = SettingsService::new(FailingSaveRepository(Settings::default()));
        let login_item = FakeLoginItem::default();

        assert!(service.update(launching_on_login(), &login_item).is_err());
        assert_eq!(*login_item.applied.borrow(), [true, false]);
    }

    #[test]
    fn sync_turns_the_setting_off_when_disabled_in_startup_apps() {
        let service = service();
        service
            .update(launching_on_login(), &FakeLoginItem::default())
            .unwrap();
        let login_item = disabled_in_startup_apps();

        let synced = service.sync_login_item(&login_item).unwrap();

        assert!(!synced.launch_on_login);
        assert!(!service.get().unwrap().launch_on_login);
        assert_eq!(*login_item.applied.borrow(), [false]);
    }

    #[test]
    fn sync_survives_a_login_item_the_os_refuses() {
        let login_item = FakeLoginItem {
            refuses: true,
            ..FakeLoginItem::default()
        };

        let synced = service().sync_login_item(&login_item).unwrap();

        assert_eq!(synced, Settings::default());
    }
}
