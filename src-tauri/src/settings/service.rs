use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Local, TimeDelta, TimeZone, Utc};

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

    pub fn update(&self, requested: Settings) -> Result<Settings, String> {
        requested.validate()?;
        self.modify(|settings| {
            // Pause only changes through pause/resume, so a stale settings form can't undo it.
            let pause = settings.pause.take();
            *settings = requested;
            settings.pause = pause;
        })?;
        self.get()
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
    use chrono::NaiveTime;

    use super::*;
    use crate::settings::repository::SqliteSettingsRepository;

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

        service.update(requested.clone()).unwrap();

        assert_eq!(service.get().unwrap(), requested);
    }

    #[test]
    fn rejects_invalid_values_without_saving() {
        let service = service();
        let mut invalid_interval = Settings::default();
        invalid_interval.categories.posture.interval_minutes = 7;
        let mut invalid_time = Settings::default();
        invalid_time.quiet_hours.end = "08:15".into();

        assert!(service.update(invalid_interval).is_err());
        assert!(service.update(invalid_time).is_err());
        assert_eq!(service.get().unwrap(), Settings::default());
    }

    #[test]
    fn pause_today_ends_at_next_local_midnight() {
        let now = Utc::now();
        let settings = service().pause_at(PauseKind::Today, now).unwrap();
        let until = settings.pause.unwrap().until;

        assert!(until > now);
        assert!(until - now <= TimeDelta::days(1));
        assert_eq!(until.with_timezone(&Local).time(), NaiveTime::MIN);
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

        let updated = service.update(Settings::default()).unwrap();

        assert_eq!(
            updated.pause.map(|pause| pause.kind),
            Some(PauseKind::OneHour)
        );
    }
}
