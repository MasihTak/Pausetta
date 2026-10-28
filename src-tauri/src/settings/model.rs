use chrono::{DateTime, NaiveTime, ParseError, Timelike, Utc};
use serde::{Deserialize, Serialize};

use super::error::SettingsError;

pub const INTERVAL_OPTIONS_MINUTES: [u32; 5] = [15, 20, 30, 45, 60];
pub const IDLE_OPTIONS_MINUTES: [u32; 5] = [3, 5, 10, 15, 30];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CategoryKey {
    Eye,
    Posture,
    Movement,
    Hydration,
}

impl CategoryKey {
    pub const ALL: [CategoryKey; 4] = [
        CategoryKey::Eye,
        CategoryKey::Posture,
        CategoryKey::Movement,
        CategoryKey::Hydration,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            CategoryKey::Eye => "eye",
            CategoryKey::Posture => "posture",
            CategoryKey::Movement => "movement",
            CategoryKey::Hydration => "hydration",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        CategoryKey::ALL
            .into_iter()
            .find(|category| category.as_str() == key)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorySettings {
    pub enabled: bool,
    pub interval_minutes: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Categories {
    pub eye: CategorySettings,
    pub posture: CategorySettings,
    pub movement: CategorySettings,
    pub hydration: CategorySettings,
}

impl Categories {
    pub fn get(&self, key: CategoryKey) -> &CategorySettings {
        match key {
            CategoryKey::Eye => &self.eye,
            CategoryKey::Posture => &self.posture,
            CategoryKey::Movement => &self.movement,
            CategoryKey::Hydration => &self.hydration,
        }
    }

    pub fn get_mut(&mut self, key: CategoryKey) -> &mut CategorySettings {
        match key {
            CategoryKey::Eye => &mut self.eye,
            CategoryKey::Posture => &mut self.posture,
            CategoryKey::Movement => &mut self.movement,
            CategoryKey::Hydration => &mut self.hydration,
        }
    }
}

impl Default for Categories {
    fn default() -> Self {
        let enabled_every = |interval_minutes| CategorySettings {
            enabled: true,
            interval_minutes,
        };
        Self {
            eye: enabled_every(20),
            posture: enabled_every(30),
            movement: enabled_every(45),
            hydration: enabled_every(60),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuietHours {
    pub enabled: bool,
    /// Local wall-clock times, on a half hour. "HH:MM" in JSON and in the database.
    #[serde(with = "time_of_day_text")]
    pub start: NaiveTime,
    #[serde(with = "time_of_day_text")]
    pub end: NaiveTime,
}

impl Default for QuietHours {
    fn default() -> Self {
        Self {
            enabled: true,
            start: time_of_day(22, 0),
            end: time_of_day(8, 0),
        }
    }
}

const TIME_OF_DAY_FORMAT: &str = "%H:%M";

pub fn time_of_day(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).expect("callers pass a valid hour and minute")
}

pub fn format_time_of_day(time: &NaiveTime) -> String {
    time.format(TIME_OF_DAY_FORMAT).to_string()
}

pub fn parse_time_of_day(text: &str) -> Result<NaiveTime, ParseError> {
    NaiveTime::parse_from_str(text, TIME_OF_DAY_FORMAT)
}

mod time_of_day_text {
    use chrono::NaiveTime;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(time: &NaiveTime, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&super::format_time_of_day(time))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveTime, D::Error> {
        let text = String::deserialize(deserializer)?;
        super::parse_time_of_day(&text).map_err(Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdlePause {
    pub enabled: bool,
    pub minutes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PauseKind {
    OneHour,
    Today,
}

impl PauseKind {
    pub const ALL: [PauseKind; 2] = [PauseKind::OneHour, PauseKind::Today];

    pub fn as_str(&self) -> &'static str {
        match self {
            PauseKind::OneHour => "oneHour",
            PauseKind::Today => "today",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        PauseKind::ALL.into_iter().find(|kind| kind.as_str() == key)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PauseState {
    pub kind: PauseKind,
    pub until: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub categories: Categories,
    pub quiet_hours: QuietHours,
    pub idle_pause: IdlePause,
    pub launch_on_login: bool,
    pub sound_enabled: bool,
    pub pause: Option<PauseState>,
    /// False until the user finishes (or skips) the first-run walkthrough.
    pub onboarding_completed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            categories: Categories::default(),
            quiet_hours: QuietHours::default(),
            idle_pause: IdlePause {
                enabled: true,
                minutes: 5,
            },
            launch_on_login: false,
            sound_enabled: false,
            pause: None,
            onboarding_completed: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), SettingsError> {
        for key in CategoryKey::ALL {
            let interval_minutes = self.categories.get(key).interval_minutes;
            if !INTERVAL_OPTIONS_MINUTES.contains(&interval_minutes) {
                return Err(SettingsError::Validation(format!(
                    "{} interval must be one of {INTERVAL_OPTIONS_MINUTES:?} minutes",
                    key.as_str()
                )));
            }
        }

        if !IDLE_OPTIONS_MINUTES.contains(&self.idle_pause.minutes) {
            return Err(SettingsError::Validation(format!(
                "idle pause must be one of {IDLE_OPTIONS_MINUTES:?} minutes"
            )));
        }

        for time in [self.quiet_hours.start, self.quiet_hours.end] {
            if !is_half_hour(time) {
                return Err(SettingsError::Validation(format!(
                    "quiet hours time '{}' must be on a half hour",
                    format_time_of_day(&time)
                )));
            }
        }

        Ok(())
    }
}

fn is_half_hour(time: NaiveTime) -> bool {
    time.minute().is_multiple_of(30) && time.second() == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_hours_keep_their_hh_mm_json_shape() {
        let json = serde_json::to_string(&QuietHours::default()).unwrap();

        assert_eq!(json, r#"{"enabled":true,"start":"22:00","end":"08:00"}"#);
        assert_eq!(
            serde_json::from_str::<QuietHours>(&json).unwrap(),
            QuietHours::default()
        );
    }

    #[test]
    fn quiet_hours_reject_a_time_that_is_not_hh_mm() {
        let json = r#"{"enabled":true,"start":"99:99","end":"08:00"}"#;

        assert!(serde_json::from_str::<QuietHours>(json).is_err());
    }

    #[test]
    fn validation_rejects_a_time_off_the_half_hour() {
        let mut settings = Settings::default();
        settings.quiet_hours.end = time_of_day(8, 15);

        assert!(matches!(
            settings.validate(),
            Err(SettingsError::Validation(_))
        ));
    }

    #[test]
    fn every_pause_kind_round_trips_through_its_key() {
        for kind in PauseKind::ALL {
            assert_eq!(PauseKind::from_key(kind.as_str()), Some(kind));
        }
    }
}
