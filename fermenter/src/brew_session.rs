//! Pure logic for the `brew-session` capability: validating an
//! operator-submitted brew identifier, persisting/restoring it alongside the
//! target temperature, and seeding the initial brew identifier from
//! persisted state on startup. Kept free of I/O beyond `TimeStore` (mirrors
//! `temperature_control.rs`'s shape per design.md decision 5) so it's
//! exhaustively unit-testable without a serial source, store, or web server.

use crate::error::Result;
use crate::store::TimeStore;

/// Parse and validate an operator-submitted brew identifier. Returns the
/// trimmed value if it is non-empty after trimming surrounding whitespace,
/// otherwise a human-readable error message (design.md decision 6).
pub fn validate_brew_id(input: &str) -> std::result::Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("brew identifier must not be empty".to_string());
    }
    Ok(trimmed.to_string())
}

/// Compute the initial brew identifier to seed the `watch` channel with on
/// startup: the persisted `ControllerState.brew_id` if present, otherwise
/// `default`. Mirrors `temperature_control::initial_target`'s startup-
/// rehydration pattern for the brew-identifier half of state.
pub async fn initial_brew_id(store: &dyn TimeStore, default: &str) -> String {
    match store.load_state().await {
        Ok(Some(state)) => state.brew_id,
        Ok(None) => default.to_string(),
        Err(_) => default.to_string(),
    }
}

/// Persist the new brew identifier paired with `current_target_temp` as the
/// `ControllerState` (brew-session: "Persist and restore the brew identifier
/// across restarts"). `current_target_temp` is sourced by the caller from
/// the live `target_tx` channel rather than read back through the store,
/// resolving slice-4's design.md open question about the two mutating
/// handlers clobbering each other's field (design.md decision 2).
pub async fn persist_brew(
    store: &dyn TimeStore,
    brew_id: &str,
    current_target_temp: f64,
) -> Result<()> {
    store
        .save_state(&crate::model::ControllerState {
            target_temp: current_target_temp,
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
    #[case("01-IPA-v02", Ok("01-IPA-v02".to_string()))]
    #[case("  01-IPA-v02  ", Ok("01-IPA-v02".to_string()))]
    #[case("a", Ok("a".to_string()))]
    fn valid_brew_ids_are_accepted_and_trimmed(
        #[case] input: &str,
        #[case] expected: std::result::Result<String, String>,
    ) {
        assert_eq!(validate_brew_id(input), expected);
    }

    #[rstest]
    #[case("")]
    #[case("   ")]
    #[case("\t\n")]
    fn empty_or_whitespace_brew_ids_are_rejected(#[case] input: &str) {
        assert!(validate_brew_id(input).is_err());
    }

    #[tokio::test]
    async fn initial_brew_id_uses_persisted_value_when_present() {
        let store = FakeTimeStore::new();
        store
            .save_state(&ControllerState {
                target_temp: 19.5,
                brew_id: "brew-1".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(initial_brew_id(&store, "00-TEST-v00").await, "brew-1");
    }

    #[tokio::test]
    async fn initial_brew_id_falls_back_to_default_when_nothing_persisted() {
        let store = FakeTimeStore::new();

        assert_eq!(initial_brew_id(&store, "00-TEST-v00").await, "00-TEST-v00");
    }

    #[tokio::test]
    async fn persist_brew_saves_brew_id_with_current_target_temp() {
        let store = FakeTimeStore::new();

        persist_brew(&store, "brew-2", 21.5).await.unwrap();

        let loaded = store.load_state().await.unwrap().unwrap();
        assert_eq!(loaded.brew_id, "brew-2");
        assert_eq!(loaded.target_temp, 21.5);
    }
}
