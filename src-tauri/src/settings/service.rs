use std::sync::{Mutex, MutexGuard, PoisonError};

use chrono::{DateTime, Local, TimeDelta, TimeZone, Utc};

use crate::autostart::LoginItem;

use super::error::SettingsError;
use super::model::{PauseKind, PauseState, Settings};
use super::repository::SettingsRepository;

pub struct SettingsService<R: SettingsRepository> {
    repository: R,
    /// The settings as last saved, so reads never touch the database. Also serializes
    /// load → change → save, so concurrent writers can't revert each other.
    current: Mutex<Settings>,
}

impl<R: SettingsRepository> SettingsService<R> {
    pub fn new(repository: R) -> Result<Self, SettingsError> {
        let current = repository.load()?;
        Ok(Self {
            repository,
            current: Mutex::new(current),
        })
    }

    pub fn get(&self) -> Settings {
        self.lock().clone()
    }

    pub fn update(
        &self,
        requested: Settings,
        login_item: &impl LoginItem,
    ) -> Result<Settings, SettingsError> {
        requested.validate()?;
        let mut current = self.lock();
        let mut settings = requested;
        // Switching the toggle on is a fresh request; an unrelated save must not undo a
        // Startup apps "Disabled".
        if is_held_back_by_user(&current, login_item) {
            settings.launch_on_login = false;
        }
        // OS first: if it refuses the login item, nothing is saved.
        login_item
            .apply(settings.launch_on_login)
            .map_err(SettingsError::Autostart)?;
        // Pause only changes through pause/resume, so a stale settings form can't undo it.
        settings.pause = current.pause.clone();
        if let Err(save_error) = self.repository.save(&settings) {
            if let Err(restore_error) = login_item.apply(current.launch_on_login) {
                log::error!("could not restore the login item: {restore_error}");
            }
            return Err(save_error);
        }
        *current = settings.clone();
        Ok(settings)
    }

    /// Startup counterpart of `update`: honours a Startup apps "Disabled", then repairs the
    /// OS login item so it matches the saved setting.
    pub fn sync_login_item(&self, login_item: &impl LoginItem) -> Result<Settings, SettingsError> {
        let mut settings = self.get();
        if is_held_back_by_user(&settings, login_item) {
            settings = self.modify(|settings| settings.launch_on_login = false)?;
        }
        // Non-fatal: a broken login item must not stop the reminders.
        if let Err(error) = login_item.apply(settings.launch_on_login) {
            log::error!("could not sync the login item: {error}");
        }
        Ok(settings)
    }

    pub fn pause(&self, kind: PauseKind) -> Result<Settings, SettingsError> {
        self.pause_at(kind, Utc::now())
    }

    pub fn resume(&self) -> Result<Settings, SettingsError> {
        self.modify(|settings| settings.pause = None)
    }

    /// The settings with an elapsed pause cleared. The scheduler calls this each tick, so
    /// the database is only written on the tick a pause actually ends.
    pub fn expire_pause(&self) -> Result<Settings, SettingsError> {
        self.expire_pause_at(Utc::now())
    }

    fn expire_pause_at(&self, now: DateTime<Utc>) -> Result<Settings, SettingsError> {
        let mut current = self.lock();
        if has_expired_pause(&current, now) {
            self.save_change(&mut current, |settings| settings.pause = None)?;
        }
        Ok(current.clone())
    }

    fn pause_at(&self, kind: PauseKind, now: DateTime<Utc>) -> Result<Settings, SettingsError> {
        let until = match kind {
            PauseKind::OneHour => now + TimeDelta::hours(1),
            PauseKind::Today => next_local_midnight(now),
        };
        self.modify(|settings| settings.pause = Some(PauseState { kind, until }))
    }

    fn modify(&self, change: impl FnOnce(&mut Settings)) -> Result<Settings, SettingsError> {
        let mut current = self.lock();
        self.save_change(&mut current, change)?;
        Ok(current.clone())
    }

    /// Saves first and only then updates `current`, so a failed save leaves both unchanged.
    fn save_change(
        &self,
        current: &mut Settings,
        change: impl FnOnce(&mut Settings),
    ) -> Result<(), SettingsError> {
        let mut changed = current.clone();
        change(&mut changed);
        self.repository.save(&changed)?;
        *current = changed;
        Ok(())
    }

    /// A panic while holding the lock leaves a complete `Settings` behind, and the scheduler
    /// reads them every tick, so carrying on beats failing every call for the rest of the session.
    fn lock(&self) -> MutexGuard<'_, Settings> {
        self.current.lock().unwrap_or_else(PoisonError::into_inner)
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    use chrono::NaiveTime;

    use super::*;
    use crate::settings::model::time_of_day;
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

    fn disk_full() -> SettingsError {
        SettingsError::Database(rusqlite::Error::InvalidQuery)
    }

    struct FailingSaveRepository(Settings);

    impl SettingsRepository for FailingSaveRepository {
        fn load(&self) -> Result<Settings, SettingsError> {
            Ok(self.0.clone())
        }

        fn save(&self, _settings: &Settings) -> Result<(), SettingsError> {
            Err(disk_full())
        }
    }

    #[derive(Default)]
    struct CountingRepository {
        loads: AtomicUsize,
    }

    impl SettingsRepository for CountingRepository {
        fn load(&self) -> Result<Settings, SettingsError> {
            self.loads.fetch_add(1, Ordering::Relaxed);
            Ok(Settings::default())
        }

        fn save(&self, _settings: &Settings) -> Result<(), SettingsError> {
            Ok(())
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
        SettingsService::new(SqliteSettingsRepository::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn seeds_defaults_on_first_open() {
        assert_eq!(service().get(), Settings::default());
    }

    #[test]
    fn update_round_trips() {
        let service = service();
        let mut requested = Settings::default();
        requested.categories.eye.interval_minutes = 45;
        requested.categories.hydration.enabled = false;
        requested.quiet_hours.start = time_of_day(21, 30);
        requested.sound_enabled = true;
        requested.onboarding_completed = true;

        service
            .update(requested.clone(), &FakeLoginItem::default())
            .unwrap();

        assert_eq!(service.get(), requested);
    }

    #[test]
    fn rejects_invalid_values_without_saving() {
        let service = service();
        let mut invalid_interval = Settings::default();
        invalid_interval.categories.posture.interval_minutes = 7;
        let mut invalid_time = Settings::default();
        invalid_time.quiet_hours.end = time_of_day(8, 15);

        assert!(matches!(
            service.update(invalid_interval, &FakeLoginItem::default()),
            Err(SettingsError::Validation(_))
        ));
        assert!(matches!(
            service.update(invalid_time, &FakeLoginItem::default()),
            Err(SettingsError::Validation(_))
        ));
        assert_eq!(service.get(), Settings::default());
    }

    #[test]
    fn reads_are_served_from_memory() {
        let service = SettingsService::new(CountingRepository::default()).unwrap();

        service.get();
        service.expire_pause().unwrap();
        service.pause(PauseKind::OneHour).unwrap();
        service.get();

        assert_eq!(service.repository.loads.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn a_failed_save_leaves_the_settings_unchanged() {
        let service = SettingsService::new(FailingSaveRepository(Settings::default())).unwrap();

        assert!(service.pause(PauseKind::OneHour).is_err());
        assert_eq!(service.get(), Settings::default());
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
    fn expire_pause_clears_an_elapsed_pause() {
        let service = service();
        let now = Utc::now();
        service
            .pause_at(PauseKind::OneHour, now - TimeDelta::hours(2))
            .unwrap();

        assert_eq!(service.expire_pause_at(now).unwrap().pause, None);
        assert_eq!(service.get().pause, None);
    }

    #[test]
    fn expire_pause_keeps_an_active_pause() {
        let service = service();
        service.pause(PauseKind::OneHour).unwrap();

        assert!(service.expire_pause().unwrap().pause.is_some());
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

        assert!(matches!(
            service.update(launching_on_login(), &login_item),
            Err(SettingsError::Autostart(_))
        ));
        assert_eq!(service.get(), Settings::default());
    }

    #[test]
    fn update_restores_the_login_item_when_saving_fails() {
        let service = SettingsService::new(FailingSaveRepository(Settings::default())).unwrap();
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
        assert!(!service.get().launch_on_login);
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
