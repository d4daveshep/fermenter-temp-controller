use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::error::Result;
use crate::model::{ControllerState, Reading};
use crate::store::TimeStore;

/// In-memory `TimeStore` test double. Records the most recent reading
/// written per brew and the most recently saved controller state, with no
/// network or Docker dependency.
#[derive(Default)]
pub struct FakeTimeStore {
    readings: Mutex<HashMap<String, Reading>>,
    state: Mutex<Option<ControllerState>>,
}

impl FakeTimeStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pre-populate as if `reading` were the most recently persisted value
    /// for `brew_id`, without going through `write_reading`. Used to set up
    /// startup-rehydration tests.
    pub fn seed_reading(&self, brew_id: &str, reading: Reading) {
        self.readings
            .lock()
            .unwrap()
            .insert(brew_id.to_string(), reading);
    }
}

#[async_trait]
impl TimeStore for FakeTimeStore {
    async fn write_reading(&self, brew_id: &str, reading: &Reading) -> Result<()> {
        self.readings
            .lock()
            .unwrap()
            .insert(brew_id.to_string(), reading.clone());
        Ok(())
    }

    async fn last_reading(&self, brew_id: &str) -> Result<Option<Reading>> {
        Ok(self.readings.lock().unwrap().get(brew_id).cloned())
    }

    async fn save_state(&self, state: &ControllerState) -> Result<()> {
        *self.state.lock().unwrap() = Some(state.clone());
        Ok(())
    }

    async fn load_state(&self) -> Result<Option<ControllerState>> {
        Ok(self.state.lock().unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(average: f64) -> Reading {
        Reading {
            target: 19.5,
            average,
            min: 18.0,
            max: 19.0,
            ambient: 20.0,
            action: "idle".to_string(),
            reason_code: String::new(),
            json_size: None,
        }
    }

    #[tokio::test]
    async fn write_then_last_reading_roundtrip() {
        let store = FakeTimeStore::new();
        store.write_reading("brew-1", &reading(19.0)).await.unwrap();

        let last = store.last_reading("brew-1").await.unwrap();
        assert_eq!(last.expect("should have a reading").average, 19.0);
    }

    #[tokio::test]
    async fn last_reading_on_empty_brew_returns_none() {
        let store = FakeTimeStore::new();
        assert!(store.last_reading("no-such-brew").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn save_then_load_state_roundtrip() {
        let store = FakeTimeStore::new();
        let state = ControllerState {
            target_temp: 19.5,
            brew_id: "brew-1".to_string(),
        };
        store.save_state(&state).await.unwrap();

        let loaded = store.load_state().await.unwrap();
        assert_eq!(loaded, Some(state));
    }

    #[tokio::test]
    async fn load_state_with_no_prior_save_returns_none() {
        let store = FakeTimeStore::new();
        assert!(store.load_state().await.unwrap().is_none());
    }
}
