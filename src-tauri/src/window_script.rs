use serde_json::{json, Value};

use crate::scheduler::service::SNOOZE_MINUTES;
use crate::settings::model::{IDLE_OPTIONS_MINUTES, INTERVAL_OPTIONS_MINUTES};

/// A script that sets each name as a `window` global before the page runs. Values go through
/// JSON so a quote or backslash in one can never end the literal and run as script.
pub fn initialization_script(globals: &[(&str, Value)]) -> String {
    globals
        .iter()
        .map(|(name, value)| format!("window.{name} = {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The numbers Rust owns and the frontend shows, so the two can't drift apart.
pub fn options_global() -> (&'static str, Value) {
    (
        "__PAUSETTA_OPTIONS__",
        json!({
            "intervalMinutes": INTERVAL_OPTIONS_MINUTES,
            "idleMinutes": IDLE_OPTIONS_MINUTES,
            "snoozeMinutes": SNOOZE_MINUTES,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_escaped_as_json_literals() {
        let script = initialization_script(&[("__NAME__", json!("a\"; alert(1); \""))]);

        assert_eq!(script, r#"window.__NAME__ = "a\"; alert(1); \"";"#);
    }

    #[test]
    fn several_globals_become_several_statements() {
        let script = initialization_script(&[("A", json!("x")), ("B", json!(2))]);

        assert_eq!(script, r#"window.A = "x"; window.B = 2;"#);
    }

    #[test]
    fn options_carry_the_rust_constants() {
        let (name, options) = options_global();

        assert_eq!(name, "__PAUSETTA_OPTIONS__");
        assert_eq!(options["intervalMinutes"], json!(INTERVAL_OPTIONS_MINUTES));
        assert_eq!(options["idleMinutes"], json!(IDLE_OPTIONS_MINUTES));
        assert_eq!(options["snoozeMinutes"], json!(SNOOZE_MINUTES));
    }
}
