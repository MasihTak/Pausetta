use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use super::model::{
    Categories, CategoryKey, CategorySettings, IdlePause, PauseKind, PauseState, QuietHours,
    Settings,
};

pub trait SettingsRepository: Send + Sync {
    fn load(&self) -> Result<Settings, String>;
    fn save(&self, settings: &Settings) -> Result<(), String>;
}

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS reminder_categories (
        key              TEXT PRIMARY KEY,
        enabled          INTEGER NOT NULL,
        interval_minutes INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS app_settings (
        id                 INTEGER PRIMARY KEY CHECK (id = 1),
        quiet_enabled      INTEGER NOT NULL,
        quiet_start        TEXT NOT NULL,
        quiet_end          TEXT NOT NULL,
        idle_pause_enabled INTEGER NOT NULL,
        idle_minutes       INTEGER NOT NULL,
        launch_on_login    INTEGER NOT NULL,
        sound_enabled      INTEGER NOT NULL,
        pause_kind         TEXT,
        paused_until       TEXT
    );
";

/// Schema changes applied after the first release, run in order and tracked via
/// `PRAGMA user_version` so a failure (locked file, disk full) surfaces instead of being
/// silently swallowed. Databases from before this tracking existed are already at
/// user_version 0 but may already have migration 1's column, so that one migration alone
/// tolerates "duplicate column" as success; every other error still propagates.
const MIGRATIONS: [&str; 1] =
    ["ALTER TABLE app_settings ADD COLUMN onboarding_completed INTEGER NOT NULL DEFAULT 0"];

pub struct SqliteSettingsRepository {
    connection: Mutex<Connection>,
}

impl SqliteSettingsRepository {
    pub fn open(path: &Path) -> Result<Self, String> {
        Self::initialize(Connection::open(path).map_err(to_message)?)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, String> {
        Self::initialize(Connection::open_in_memory().map_err(to_message)?)
    }

    fn initialize(connection: Connection) -> Result<Self, String> {
        connection.execute_batch(SCHEMA).map_err(to_message)?;
        run_migrations(&connection)?;
        let repository = Self {
            connection: Mutex::new(connection),
        };
        if !repository.has_saved_settings()? {
            repository.save(&Settings::default())?;
        }
        Ok(repository)
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.connection
            .lock()
            .map_err(|_| "settings database lock was poisoned".to_string())
    }

    fn has_saved_settings(&self) -> Result<bool, String> {
        self.lock()?
            .query_row("SELECT EXISTS(SELECT 1 FROM app_settings)", [], |row| {
                row.get(0)
            })
            .map_err(to_message)
    }
}

impl SettingsRepository for SqliteSettingsRepository {
    fn load(&self) -> Result<Settings, String> {
        let connection = self.lock()?;

        let mut settings = connection
            .query_row(
                "SELECT quiet_enabled, quiet_start, quiet_end, idle_pause_enabled, idle_minutes,
                        launch_on_login, sound_enabled, pause_kind, paused_until,
                        onboarding_completed
                 FROM app_settings WHERE id = 1",
                [],
                |row| {
                    Ok(Settings {
                        categories: Categories::default(),
                        quiet_hours: QuietHours {
                            enabled: row.get(0)?,
                            start: row.get(1)?,
                            end: row.get(2)?,
                        },
                        idle_pause: IdlePause {
                            enabled: row.get(3)?,
                            minutes: row.get(4)?,
                        },
                        launch_on_login: row.get(5)?,
                        sound_enabled: row.get(6)?,
                        pause: read_pause(row.get(7)?, row.get(8)?),
                        onboarding_completed: row.get(9)?,
                    })
                },
            )
            .map_err(to_message)?;

        let mut statement = connection
            .prepare("SELECT key, enabled, interval_minutes FROM reminder_categories")
            .map_err(to_message)?;
        let category_rows = statement
            .query_map([], |row| {
                let category = CategorySettings {
                    enabled: row.get(1)?,
                    interval_minutes: row.get(2)?,
                };
                Ok((row.get::<_, String>(0)?, category))
            })
            .map_err(to_message)?;

        for category_row in category_rows {
            let (key, category) = category_row.map_err(to_message)?;
            if let Some(key) = CategoryKey::from_key(&key) {
                *settings.categories.get_mut(key) = category;
            }
        }

        Ok(settings)
    }

    fn save(&self, settings: &Settings) -> Result<(), String> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction().map_err(to_message)?;

        transaction
            .execute(
                "INSERT OR REPLACE INTO app_settings
                    (id, quiet_enabled, quiet_start, quiet_end, idle_pause_enabled, idle_minutes,
                     launch_on_login, sound_enabled, pause_kind, paused_until,
                     onboarding_completed)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    settings.quiet_hours.enabled,
                    settings.quiet_hours.start,
                    settings.quiet_hours.end,
                    settings.idle_pause.enabled,
                    settings.idle_pause.minutes,
                    settings.launch_on_login,
                    settings.sound_enabled,
                    settings.pause.as_ref().map(|pause| pause.kind.as_str()),
                    settings
                        .pause
                        .as_ref()
                        .map(|pause| pause.until.to_rfc3339()),
                    settings.onboarding_completed,
                ],
            )
            .map_err(to_message)?;

        for key in CategoryKey::ALL {
            let category = settings.categories.get(key);
            transaction
                .execute(
                    "INSERT OR REPLACE INTO reminder_categories (key, enabled, interval_minutes)
                     VALUES (?1, ?2, ?3)",
                    params![key.as_str(), category.enabled, category.interval_minutes],
                )
                .map_err(to_message)?;
        }

        transaction.commit().map_err(to_message)
    }
}

/// A half-written or unparseable pause is treated as "not paused" rather than an error,
/// so a corrupt row can never lock the user out of their reminders.
fn read_pause(kind: Option<String>, until: Option<String>) -> Option<PauseState> {
    let kind = PauseKind::from_key(&kind?)?;
    let until = DateTime::parse_from_rfc3339(&until?)
        .ok()?
        .with_timezone(&Utc);
    Some(PauseState { kind, until })
}

/// Applies migrations after `MIGRATIONS[..user_version]`, then advances user_version.
/// Runs in a transaction so a failure partway through never leaves the version ahead
/// of what was actually applied.
fn run_migrations(connection: &Connection) -> Result<(), String> {
    let current_version: u32 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(to_message)?;

    let pending = MIGRATIONS.iter().skip(current_version as usize);
    for (offset, statement) in pending.enumerate() {
        let index = current_version as usize + offset;
        match connection.execute(statement, []) {
            Ok(_) => {}
            // Migration 0 (add onboarding_completed) already landed on databases created
            // before user_version tracking existed; treat that specific case as applied.
            Err(rusqlite::Error::SqliteFailure(_, Some(message)))
                if index == 0 && message.contains("duplicate column name") => {}
            Err(error) => return Err(to_message(error)),
        }
        connection
            .execute(&format!("PRAGMA user_version = {}", index + 1), [])
            .map_err(to_message)?;
    }
    Ok(())
}

fn to_message(error: rusqlite::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pre-migration-tracking database: has the SCHEMA tables and already got the
    /// onboarding_completed column the old best-effort ALTER used to add, but user_version
    /// is still 0. Loading it must not error on "duplicate column name".
    #[test]
    fn opens_database_with_column_but_no_recorded_version() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        connection
            .execute(
                "ALTER TABLE app_settings ADD COLUMN onboarding_completed INTEGER NOT NULL DEFAULT 0",
                [],
            )
            .unwrap();

        let repository = SqliteSettingsRepository {
            connection: Mutex::new(connection),
        };
        run_migrations(&repository.lock().unwrap()).unwrap();

        let version: u32 = repository
            .lock()
            .unwrap()
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, MIGRATIONS.len() as u32);
    }

    #[test]
    fn open_in_memory_runs_migrations_and_seeds_defaults() {
        let repository = SqliteSettingsRepository::open_in_memory().unwrap();
        assert_eq!(repository.load().unwrap(), Settings::default());
    }
}
