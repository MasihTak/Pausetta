use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    // Defaults follow science-research.md: 20-20-20 for eyes, movement in the
    // well-supported 30–60 min range, posture paired near it, hydration hourly.
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
    /// Local wall-clock time, "HH:MM" on a half hour.
    pub start: String,
    pub end: String,
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
    pub fn as_str(&self) -> &'static str {
        match self {
            PauseKind::OneHour => "oneHour",
            PauseKind::Today => "today",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        [PauseKind::OneHour, PauseKind::Today]
            .into_iter()
            .find(|kind| kind.as_str() == key)
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
            quiet_hours: QuietHours {
                enabled: true,
                start: "22:00".into(),
                end: "08:00".into(),
            },
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
    pub fn validate(&self) -> Result<(), String> {
        for key in CategoryKey::ALL {
            let interval_minutes = self.categories.get(key).interval_minutes;
            if !INTERVAL_OPTIONS_MINUTES.contains(&interval_minutes) {
                return Err(format!(
                    "{} interval must be one of {INTERVAL_OPTIONS_MINUTES:?} minutes",
                    key.as_str()
                ));
            }
        }

        if !IDLE_OPTIONS_MINUTES.contains(&self.idle_pause.minutes) {
            return Err(format!(
                "idle pause must be one of {IDLE_OPTIONS_MINUTES:?} minutes"
            ));
        }

        for time in [&self.quiet_hours.start, &self.quiet_hours.end] {
            if !is_half_hour_time(time) {
                return Err(format!(
                    "quiet hours time '{time}' must be HH:MM on a half hour"
                ));
            }
        }

        Ok(())
    }
}

fn is_half_hour_time(value: &str) -> bool {
    let Some((hours, minutes)) = value.split_once(':') else {
        return false;
    };
    let is_valid_hour = hours.len() == 2 && hours.parse::<u8>().is_ok_and(|hour| hour < 24);
    let is_half_hour_minute = minutes == "00" || minutes == "30";
    is_valid_hour && is_half_hour_minute
}
