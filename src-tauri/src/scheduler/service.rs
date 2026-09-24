use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use chrono::{DateTime, Local, TimeDelta, Timelike, Utc};

use super::idle::IdleDetector;
use crate::settings::model::{CategoryKey, Settings};

/// Past this, a rule was missed (the machine slept) and skips to its next cycle instead of
/// firing late. Generous because the OS can throttle a background app's timer (App Nap).
const OVERDUE_GRACE: TimeDelta = TimeDelta::minutes(5);

/// A second rule coming due in the same tick waits this long rather than being skipped to
/// its next cycle. Two toasts a minute apart is fine; two at once is not — and skipping
/// would starve any category whose interval is a multiple of another's (hydration at 60
/// minutes would collide with eye care at 20 every single time and never fire).
const COLLISION_DELAY: TimeDelta = TimeDelta::minutes(1);

/// How far a snoozed reminder is pushed back before its normal interval resumes.
pub const SNOOZE_MINUTES: i64 = 10;

/// When a category fires next, and the interval that time was derived from. Storing the
/// interval is what makes "the user just changed this setting" detectable without the
/// scheduler having to subscribe to settings changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Trigger {
    at: DateTime<Utc>,
    interval_minutes: u32,
}

#[derive(Default)]
struct SchedulerState {
    triggers: HashMap<CategoryKey, Trigger>,
    last_tick_at: Option<DateTime<Utc>>,
}

/// Owns *when* each reminder fires. Timestamps, never countdowns: comparing `now` against
/// a stored instant is what makes sleep, wake, and long uptime correct for free.
pub struct SchedulerService<D: IdleDetector> {
    idle_detector: D,
    state: Mutex<SchedulerState>,
}

impl<D: IdleDetector> SchedulerService<D> {
    pub fn new(idle_detector: D) -> Self {
        Self {
            idle_detector,
            state: Mutex::new(SchedulerState::default()),
        }
    }

    /// The category to show right now, if any.
    pub fn tick(&self, settings: &Settings) -> Option<CategoryKey> {
        self.tick_at(Utc::now(), settings, self.idle_detector.idle_duration())
    }

    /// Pushes one category back by [`SNOOZE_MINUTES`]; its normal interval resumes after.
    pub fn snooze(&self, category: CategoryKey) {
        if let Some(trigger) = self.lock().triggers.get_mut(&category) {
            trigger.at = Utc::now() + TimeDelta::minutes(SNOOZE_MINUTES);
        }
    }

    /// At most one category per tick, so two rules coming due in the same second can never
    /// stack two alerts on top of each other.
    fn tick_at(
        &self,
        now: DateTime<Utc>,
        settings: &Settings,
        idle: Duration,
    ) -> Option<CategoryKey> {
        let mut state = self.lock();

        // Triggers are wall-clock instants, so a clock set back (by hand or NTP) would
        // otherwise delay every reminder by the size of the jump.
        if let Some(jump_back) = state.last_tick_at.map(|last| last - now) {
            if jump_back > TimeDelta::zero() {
                state
                    .triggers
                    .values_mut()
                    .for_each(|trigger| trigger.at -= jump_back);
            }
        }

        // No ticks run during sleep, so the gap counts as idle time.
        let time_since_last_tick = state
            .last_tick_at
            .and_then(|last| (now - last).to_std().ok())
            .unwrap_or_default();
        state.last_tick_at = Some(now);

        // Each of these restarts the interval rather than firing late: an ignored or
        // suppressed reminder waits for its next cycle and never re-nags on return.
        let is_suppressed = settings.pause.is_some()
            || is_idle(settings, idle.max(time_since_last_tick))
            || is_within_quiet_hours(settings, now);

        let mut due = None;
        for key in CategoryKey::ALL {
            let category = settings.categories.get(key);
            if !category.enabled {
                state.triggers.remove(&key);
                continue;
            }

            let interval = TimeDelta::minutes(i64::from(category.interval_minutes));
            let trigger = state.triggers.entry(key).or_insert(Trigger {
                at: now + interval,
                interval_minutes: category.interval_minutes,
            });

            let has_new_interval = trigger.interval_minutes != category.interval_minutes;
            let was_missed = now - trigger.at > OVERDUE_GRACE;
            if has_new_interval || is_suppressed || was_missed {
                trigger.interval_minutes = category.interval_minutes;
                trigger.at = now + interval;
                continue;
            }

            if now < trigger.at {
                continue;
            }

            if due.is_some() {
                trigger.at = now + COLLISION_DELAY;
                continue;
            }

            trigger.at = now + interval;
            due = Some(key);
        }
        due
    }

    /// A poisoned lock would stop reminders for the rest of the session, which is a worse
    /// outcome than carrying on with the state the panicking tick left behind.
    fn lock(&self) -> MutexGuard<'_, SchedulerState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn is_idle(settings: &Settings, idle: Duration) -> bool {
    settings.idle_pause.enabled && idle.as_secs() >= u64::from(settings.idle_pause.minutes) * 60
}

/// Quiet hours are local wall-clock times and usually wrap past midnight (22:00 to 08:00).
fn is_within_quiet_hours(settings: &Settings, now: DateTime<Utc>) -> bool {
    let quiet_hours = &settings.quiet_hours;
    if !quiet_hours.enabled {
        return false;
    }

    let (Some(start), Some(end)) = (
        minutes_of_day(&quiet_hours.start),
        minutes_of_day(&quiet_hours.end),
    ) else {
        return false;
    };
    // Equal bounds are an empty window, not a silent all-day one.
    if start == end {
        return false;
    }

    let local_now = now.with_timezone(&Local);
    let current = local_now.hour() * 60 + local_now.minute();
    if start < end {
        (start..end).contains(&current)
    } else {
        current >= start || current < end
    }
}

fn minutes_of_day(value: &str) -> Option<u32> {
    let (hours, minutes) = value.split_once(':')?;
    Some(hours.parse::<u32>().ok()? * 60 + minutes.parse::<u32>().ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::model::{PauseKind, PauseState};

    struct NeverIdle;

    impl IdleDetector for NeverIdle {
        fn idle_duration(&self) -> Duration {
            Duration::ZERO
        }
    }

    type TestScheduler = SchedulerService<NeverIdle>;

    fn scheduler() -> TestScheduler {
        SchedulerService::new(NeverIdle)
    }

    /// Only eye care on, every 20 minutes, with nothing else able to suppress it.
    fn eye_only() -> Settings {
        let mut settings = Settings::default();
        settings.categories.posture.enabled = false;
        settings.categories.movement.enabled = false;
        settings.categories.hydration.enabled = false;
        settings.quiet_hours.enabled = false;
        settings.idle_pause.enabled = false;
        settings
    }

    /// The first tick can only ever schedule; it returns the instant it happened.
    fn start(scheduler: &TestScheduler, settings: &Settings) -> DateTime<Utc> {
        let start = Utc::now();
        assert_eq!(scheduler.tick_at(start, settings, Duration::ZERO), None);
        start
    }

    /// Drives the scheduler the way the real loop does — one tick a second — and reports
    /// how many minutes after `from` each reminder fired. Ticking for real is what keeps
    /// these tests honest: jumping the clock forward would trip the sleep detection.
    fn run_for(
        scheduler: &TestScheduler,
        settings: &Settings,
        from: DateTime<Utc>,
        minutes: i64,
        idle: Duration,
    ) -> Vec<(i64, CategoryKey)> {
        let mut fired = Vec::new();
        for second in 1..=(minutes * 60) {
            let now = from + TimeDelta::seconds(second);
            if let Some(category) = scheduler.tick_at(now, settings, idle) {
                fired.push((second / 60, category));
            }
        }
        fired
    }

    fn run_awake(
        scheduler: &TestScheduler,
        settings: &Settings,
        from: DateTime<Utc>,
        minutes: i64,
    ) -> Vec<(i64, CategoryKey)> {
        run_for(scheduler, settings, from, minutes, Duration::ZERO)
    }

    fn at_local_hour(hour: u32) -> DateTime<Utc> {
        Local::now()
            .with_hour(hour)
            .and_then(|time| time.with_minute(30))
            .expect("every hour has a :30")
            .with_timezone(&Utc)
    }

    #[test]
    fn fires_once_per_interval() {
        let scheduler = scheduler();
        let settings = eye_only();
        let start = start(&scheduler, &settings);

        let fired = run_awake(&scheduler, &settings, start, 45);

        assert_eq!(
            fired,
            vec![(20, CategoryKey::Eye), (40, CategoryKey::Eye)],
            "eye care should fire on the 20 minute mark and then again 20 minutes later"
        );
    }

    #[test]
    fn skips_reminders_that_came_due_while_the_machine_slept() {
        let scheduler = scheduler();
        let settings = eye_only();
        let start = start(&scheduler, &settings);

        // A four-hour sleep: twelve eye-care cycles came and went in one tick.
        let wake = start + TimeDelta::hours(4);
        assert_eq!(scheduler.tick_at(wake, &settings, Duration::ZERO), None);

        // The interval restarts from the wake instead of firing a backlog.
        assert_eq!(
            run_awake(&scheduler, &settings, wake, 21),
            vec![(20, CategoryKey::Eye)]
        );
    }

    #[test]
    fn setting_the_clock_back_does_not_delay_reminders() {
        let scheduler = scheduler();
        let settings = eye_only();
        let start = start(&scheduler, &settings);
        assert!(run_awake(&scheduler, &settings, start, 10).is_empty());

        // Ten minutes in, the clock jumps back two hours; eye care is still 10 minutes away.
        let jumped = start + TimeDelta::minutes(10) - TimeDelta::hours(2);

        assert_eq!(
            run_awake(&scheduler, &settings, jumped, 11),
            vec![(10, CategoryKey::Eye)]
        );
    }

    #[test]
    fn throttled_ticks_still_fire_reminders() {
        let scheduler = scheduler();
        let settings = eye_only();
        let start = start(&scheduler, &settings);

        let fired: Vec<i64> = (1..=90)
            .map(|step| start + TimeDelta::seconds(step * 30))
            .filter(|&now| scheduler.tick_at(now, &settings, Duration::ZERO).is_some())
            .map(|now| (now - start).num_minutes())
            .collect();

        assert_eq!(fired, vec![20, 40]);
    }

    #[test]
    fn a_sleep_longer_than_the_idle_threshold_restarts_intervals() {
        let scheduler = scheduler();
        let mut settings = eye_only();
        settings.idle_pause.enabled = true;
        settings.idle_pause.minutes = 5;
        let start = start(&scheduler, &settings);
        assert!(run_awake(&scheduler, &settings, start, 5).is_empty());

        // Asleep from minute 5 to minute 15; eye care was due at minute 20.
        let wake = start + TimeDelta::minutes(15);
        assert_eq!(scheduler.tick_at(wake, &settings, Duration::ZERO), None);

        assert_eq!(
            run_awake(&scheduler, &settings, wake, 21),
            vec![(20, CategoryKey::Eye)]
        );
    }

    #[test]
    fn idle_pauses_the_countdown_and_returning_restarts_it() {
        let scheduler = scheduler();
        let mut settings = eye_only();
        settings.idle_pause.enabled = true;
        settings.idle_pause.minutes = 5;
        let start = start(&scheduler, &settings);

        let away = Duration::from_secs(10 * 60);
        assert!(run_for(&scheduler, &settings, start, 30, away).is_empty());

        // Back at the keyboard: a full interval, not a reminder waiting at the door.
        let back = start + TimeDelta::minutes(30);
        assert_eq!(
            run_awake(&scheduler, &settings, back, 21),
            vec![(20, CategoryKey::Eye)]
        );
    }

    #[test]
    fn a_pause_suppresses_reminders() {
        let scheduler = scheduler();
        let mut settings = eye_only();
        let start = start(&scheduler, &settings);
        settings.pause = Some(PauseState {
            kind: PauseKind::OneHour,
            until: start + TimeDelta::hours(1),
        });

        assert!(run_awake(&scheduler, &settings, start, 25).is_empty());
    }

    #[test]
    fn quiet_hours_suppress_reminders() {
        let scheduler = scheduler();
        let mut settings = eye_only();
        settings.quiet_hours.enabled = true;
        settings.quiet_hours.start = "00:00".into();
        settings.quiet_hours.end = "23:30".into();

        let midday = at_local_hour(12);
        assert_eq!(scheduler.tick_at(midday, &settings, Duration::ZERO), None);
        assert!(run_awake(&scheduler, &settings, midday, 25).is_empty());
    }

    #[test]
    fn changing_an_interval_reschedules_from_now() {
        let scheduler = scheduler();
        let mut settings = eye_only();
        let start = start(&scheduler, &settings);
        assert!(run_awake(&scheduler, &settings, start, 10).is_empty());

        // Shortened to 15 minutes at the 10 minute mark: the next one is 15 from *there*,
        // not 5 minutes away on the old schedule.
        settings.categories.eye.interval_minutes = 15;
        let changed_at = start + TimeDelta::minutes(10);

        assert_eq!(
            run_awake(&scheduler, &settings, changed_at, 16),
            vec![(15, CategoryKey::Eye)]
        );
    }

    #[test]
    fn two_categories_due_together_are_spaced_out_instead_of_stacked() {
        let scheduler = scheduler();
        let mut settings = eye_only();
        settings.categories.posture.enabled = true;
        settings.categories.posture.interval_minutes = 20;
        let start = start(&scheduler, &settings);

        assert_eq!(
            run_awake(&scheduler, &settings, start, 25),
            vec![(20, CategoryKey::Eye), (21, CategoryKey::Posture)]
        );
    }

    /// The starvation case: hydration's hour is an exact multiple of eye care's 20
    /// minutes, so the two collide on every single hydration cycle.
    #[test]
    fn a_category_colliding_every_cycle_still_fires() {
        let scheduler = scheduler();
        let mut settings = eye_only();
        settings.categories.hydration.enabled = true;
        settings.categories.hydration.interval_minutes = 60;
        let start = start(&scheduler, &settings);

        assert_eq!(
            run_awake(&scheduler, &settings, start, 62),
            vec![
                (20, CategoryKey::Eye),
                (40, CategoryKey::Eye),
                (60, CategoryKey::Eye),
                (61, CategoryKey::Hydration),
            ]
        );
    }

    #[test]
    fn disabling_a_category_stops_it_firing() {
        let scheduler = scheduler();
        let mut settings = eye_only();
        let start = start(&scheduler, &settings);
        assert!(run_awake(&scheduler, &settings, start, 10).is_empty());

        settings.categories.eye.enabled = false;

        assert!(run_awake(&scheduler, &settings, start + TimeDelta::minutes(10), 30).is_empty());
    }

    #[test]
    fn snooze_pushes_the_next_trigger_back() {
        let scheduler = scheduler();
        let settings = eye_only();
        let start = start(&scheduler, &settings);
        assert_eq!(
            run_awake(&scheduler, &settings, start, 21),
            vec![(20, CategoryKey::Eye)]
        );

        scheduler.snooze(CategoryKey::Eye);

        let snoozed_until = scheduler.lock().triggers[&CategoryKey::Eye].at;
        let expected = Utc::now() + TimeDelta::minutes(SNOOZE_MINUTES);
        assert!((snoozed_until - expected).abs() < TimeDelta::seconds(5));
    }

    #[test]
    fn quiet_hours_wrap_past_midnight() {
        let mut settings = Settings::default();
        settings.quiet_hours.enabled = true;
        settings.quiet_hours.start = "22:00".into();
        settings.quiet_hours.end = "08:00".into();

        assert!(is_within_quiet_hours(&settings, at_local_hour(23)));
        assert!(is_within_quiet_hours(&settings, at_local_hour(3)));
        assert!(!is_within_quiet_hours(&settings, at_local_hour(8)));
        assert!(!is_within_quiet_hours(&settings, at_local_hour(14)));
    }

    #[test]
    fn quiet_hours_inside_one_day_do_not_wrap() {
        let mut settings = Settings::default();
        settings.quiet_hours.enabled = true;
        settings.quiet_hours.start = "09:00".into();
        settings.quiet_hours.end = "17:00".into();

        assert!(is_within_quiet_hours(&settings, at_local_hour(12)));
        assert!(!is_within_quiet_hours(&settings, at_local_hour(8)));
        assert!(!is_within_quiet_hours(&settings, at_local_hour(18)));
    }
}
