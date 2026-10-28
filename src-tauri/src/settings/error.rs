use std::fmt;

use serde::{Serialize, Serializer};

#[derive(Debug)]
pub enum SettingsError {
    /// The request was rejected; the message is meant for the user.
    Validation(String),
    Database(rusqlite::Error),
    Autostart(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::Validation(message) => formatter.write_str(message),
            SettingsError::Database(error) => write!(formatter, "settings database error: {error}"),
            SettingsError::Autostart(message) => {
                write!(formatter, "could not update the login item: {message}")
            }
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<rusqlite::Error> for SettingsError {
    fn from(error: rusqlite::Error) -> Self {
        SettingsError::Database(error)
    }
}

/// Commands hand the frontend a plain message; the kind only matters inside Rust.
impl Serialize for SettingsError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
