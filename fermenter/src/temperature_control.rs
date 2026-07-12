//! Pure logic for the `temperature-control` capability: validating an
//! operator-submitted target temperature, deciding whether a reconcile write
//! is needed, and seeding the initial target from persisted state on
//! startup. Kept free of I/O so it's exhaustively unit-testable without a
//! serial source, store, or web server.

use crate::error::Result;
use crate::store::TimeStore;

/// Minimum allowed target temperature (°C) — see design.md decision 4.
pub const TARGET_MIN: f64 = 0.0;
/// Maximum allowed target temperature (°C) — see design.md decision 4.
pub const TARGET_MAX: f64 = 40.0;

/// Parse and validate an operator-submitted target temperature string.
/// Returns the parsed value if it is finite and within
/// `TARGET_MIN..=TARGET_MAX`, otherwise a human-readable error message.
pub fn validate_target(input: &str) -> std::result::Result<f64, String> {
    let value: f64 = input
        .trim()
        .parse()
        .map_err(|_| format!("'{input}' is not a valid number"))?;

    if !value.is_finite() {
        return Err(format!("'{input}' is not a valid number"));
    }

    if !(TARGET_MIN..=TARGET_MAX).contains(&value) {
        return Err(format!(
            "target must be between {TARGET_MIN} and {TARGET_MAX}, got {value}"
        ));
    }

    Ok(value)
}

/// Decide whether the device needs a target write: `Some(desired)` when the
/// desired target differs from the target most recently reported by the
/// device, `None` when already in sync (device-connection: "write only when
/// it differs").
pub fn target_to_write(desired: f64, reported: f64) -> Option<f64> {
    if desired != reported {
        Some(desired)
    } else {
        None
    }
}

/// Compute the initial target temperature to seed the `watch` channel with
/// on startup: the persisted `ControllerState.target_temp` if present,
/// otherwise `default`. Mirrors `ingest::rehydrate_latest_reading`'s
/// startup-rehydration pattern for the target-temperature half of state.
pub async fn initial_target(store: &dyn TimeStore, default: f64) -> f64 {
    match store.load_state().await {
        Ok(Some(state)) => state.target_temp,
        Ok(None) => default,
        Err(_) => default,
    }
}

/// Persist the current target temperature paired with `brew_id` as the
/// `ControllerState` (temperature-control: "Accepted target change is
/// persisted").
pub async fn persist_target(store: &dyn TimeStore, target_temp: f64, brew_id: &str) -> Result<()> {
    store
        .save_state(&crate::model::ControllerState {
            target_temp,
            brew_id: brew_id.to_string(),
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ControllerState;
    use crate::store::fake::FakeTimeStore;
    use rstest::rstest;

    #[rstest]
    #[case("19.5", Ok(19.5))]
    #[case("0", Ok(0.0))]
    #[case("40", Ok(40.0))]
    #[case(" 20.1 ", Ok(20.1))]
    fn valid_targets_are_accepted(
        #[case] input: &str,
        #[case] expected: std::result::Result<f64, String>,
    ) {
        assert_eq!(validate_target(input), expected);
    }

    #[rstest]
    #[case("not a number")]
    #[case("")]
    #[case("NaN")]
    fn non_numeric_targets_are_rejected(#[case] input: &str) {
        assert!(validate_target(input).is_err());
    }

    #[rstest]
    #[case("-0.1")]
    #[case("40.1")]
    #[case("100")]
    fn out_of_range_targets_are_rejected(#[case] input: &str) {
        assert!(validate_target(input).is_err());
    }

    #[test]
    fn write_needed_when_desired_differs_from_reported() {
        assert_eq!(target_to_write(20.0, 19.5), Some(20.0));
    }

    #[test]
    fn no_write_needed_when_desired_matches_reported() {
        assert_eq!(target_to_write(19.5, 19.5), None);
    }

    #[tokio::test]
    async fn initial_target_uses_persisted_value_when_present() {
        let store = FakeTimeStore::new();
        store
            .save_state(&ControllerState {
                target_temp: 21.0,
                brew_id: "brew-1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(initial_target(&store, 19.5).await, 21.0);
    }

    #[tokio::test]
    async fn initial_target_falls_back_to_default_when_nothing_persisted() {
        let store = FakeTimeStore::new();

        assert_eq!(initial_target(&store, 19.5).await, 19.5);
    }

    #[tokio::test]
    async fn persist_target_saves_target_temp_with_brew_id() {
        let store = FakeTimeStore::new();

        persist_target(&store, 22.0, "brew-1").await.unwrap();

        let loaded = store.load_state().await.unwrap().unwrap();
        assert_eq!(loaded.target_temp, 22.0);
        assert_eq!(loaded.brew_id, "brew-1");
    }
}
