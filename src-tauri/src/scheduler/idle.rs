use std::time::Duration;

/// Reader interface for "how long since the user last touched this machine".
/// Keeping it a trait lets the scheduler be tested without a real desktop session.
pub trait IdleDetector: Send + Sync {
    fn idle_duration(&self) -> Duration;
}

pub struct SystemIdleDetector;

impl IdleDetector for SystemIdleDetector {
    /// A platform that can't answer (no X11 display, missing macOS Accessibility
    /// permission) reports "not idle", so reminders keep firing instead of silently
    /// stopping forever — the one failure mode this app cannot afford.
    fn idle_duration(&self) -> Duration {
        user_idle::UserIdle::get_time()
            .map(|idle| Duration::from_secs(idle.as_seconds()))
            .unwrap_or_default()
    }
}
