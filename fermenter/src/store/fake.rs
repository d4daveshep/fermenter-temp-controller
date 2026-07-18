use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};

use crate::error::Result;
use crate::model::{ControllerState, Reading};
use crate::store::{TemperatureSample, TimeStore};

/// In-memory `TimeStore` test double. Records the most recent reading
/// written per brew and the most recently saved controller state, with no
/// network or Docker dependency.
#[derive(Default)]
pub struct FakeTimeStore {
    readings: Mutex<HashMap<String, Reading>>,
    temperature_samples: Mutex<HashMap<String, Vec<TemperatureSample>>>,
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

    /// Pre-populate a complete timestamped temperature sample for `brew_id`.
    pub fn seed_temperature_sample(&self, brew_id: &str, sample: TemperatureSample) {
        self.temperature_samples
            .lock()
            .unwrap()
            .entry(brew_id.to_string())
            .or_default()
            .push(sample);
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

    async fn temperature_history(
        &self,
        brew_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        _bucket: Duration,
    ) -> Result<Vec<TemperatureSample>> {
        Ok(self
            .temperature_samples
            .lock()
            .unwrap()
            .get(brew_id)
            .into_iter()
            .flatten()
            .filter(|sample| sample.timestamp >= start && sample.timestamp <= end)
            .cloned()
            .collect())
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
    use chrono::TimeZone;

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
    async fn temperature_history_returns_complete_samples_within_the_range() {
        let store = FakeTimeStore::new();
        let before = Utc.timestamp_millis_opt(999).unwrap();
        let first = Utc.timestamp_millis_opt(1_000).unwrap();
        let second = Utc.timestamp_millis_opt(2_000).unwrap();
        let after = Utc.timestamp_millis_opt(3_001).unwrap();
        store.seed_temperature_sample(
            "brew-1",
            TemperatureSample {
                timestamp: before,
                fermenter: 17.0,
                ambient: 18.0,
                target: 19.0,
            },
        );
        store.seed_temperature_sample(
            "brew-1",
            TemperatureSample {
                timestamp: first,
                fermenter: 18.0,
                ambient: 19.0,
                target: 20.0,
            },
        );
        store.seed_temperature_sample(
            "brew-1",
            TemperatureSample {
                timestamp: second,
                fermenter: 19.0,
                ambient: 20.0,
                target: 21.0,
            },
        );
        store.seed_temperature_sample(
            "brew-1",
            TemperatureSample {
                timestamp: after,
                fermenter: 20.0,
                ambient: 21.0,
                target: 22.0,
            },
        );

        let history = store
            .temperature_history("brew-1", first, second, Duration::milliseconds(1_000))
            .await
            .unwrap();

        assert_eq!(
            history,
            vec![
                TemperatureSample {
                    timestamp: first,
                    fermenter: 18.0,
                    ambient: 19.0,
                    target: 20.0,
                },
                TemperatureSample {
                    timestamp: second,
                    fermenter: 19.0,
                    ambient: 20.0,
                    target: 21.0,
                },
            ]
        );
    }

    #[tokio::test]
    async fn temperature_history_returns_no_samples_for_an_empty_range() {
        let store = FakeTimeStore::new();
        store.seed_temperature_sample(
            "brew-1",
            TemperatureSample {
                timestamp: Utc.timestamp_millis_opt(1_000).unwrap(),
                fermenter: 18.0,
                ambient: 19.0,
                target: 20.0,
            },
        );

        let history = store
            .temperature_history(
                "brew-1",
                Utc.timestamp_millis_opt(2_000).unwrap(),
                Utc.timestamp_millis_opt(3_000).unwrap(),
                Duration::milliseconds(1_000),
            )
            .await
            .unwrap();

        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn temperature_history_excludes_another_brews_samples() {
        let store = FakeTimeStore::new();
        let timestamp = Utc.timestamp_millis_opt(1_000).unwrap();
        store.seed_temperature_sample(
            "brew-1",
            TemperatureSample {
                timestamp,
                fermenter: 18.0,
                ambient: 19.0,
                target: 20.0,
            },
        );
        store.seed_temperature_sample(
            "brew-2",
            TemperatureSample {
                timestamp,
                fermenter: 10.0,
                ambient: 11.0,
                target: 12.0,
            },
        );

        let history = store
            .temperature_history(
                "brew-1",
                timestamp,
                timestamp,
                Duration::milliseconds(1_000),
            )
            .await
            .unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].fermenter, 18.0);
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
