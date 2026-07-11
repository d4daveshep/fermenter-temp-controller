use std::sync::{Arc, Mutex};

use tracing::{info, warn};

use crate::error::AppError;
use crate::model::Reading;
use crate::serial::SerialSource;
use crate::store::TimeStore;

/// Rehydrate current state from persisted storage on startup
/// (temperature-monitoring: "Expose latest reading as current state" —
/// rehydration scenarios). Returns the most recently persisted reading for
/// `brew_id`, or `None` if nothing has been persisted for it yet, in which
/// case current state starts empty exactly as it did before this slice.
pub async fn rehydrate_latest_reading(store: &dyn TimeStore, brew_id: &str) -> Option<Reading> {
    match store.last_reading(brew_id).await {
        Ok(Some(reading)) => {
            info!(
                brew_id,
                average = reading.average,
                "current state rehydrated from storage"
            );
            Some(reading)
        }
        Ok(None) => {
            info!(
                brew_id,
                "no persisted reading found — starting with empty state"
            );
            None
        }
        Err(e) => {
            warn!(error = %e, brew_id, "failed to rehydrate state from storage — starting empty");
            None
        }
    }
}

/// Ingest loop: reads lines from `source`, parses them as `Reading` values,
/// updates `latest_reading` on success, logs a warning and continues on
/// failure. Each successfully parsed reading is also written to `store`
/// tagged with `brew_id`; a persistence failure is logged and does not stop
/// the loop (reading-history: "Persist readings..." scenario). Runs until the
/// source is exhausted or returns a terminal error.
pub async fn ingest_loop(
    source: &mut dyn SerialSource,
    latest_reading: Arc<Mutex<Option<Reading>>>,
    store: &dyn TimeStore,
    brew_id: &str,
) {
    loop {
        let line = match source.read_line().await {
            Ok(l) => l,
            Err(AppError::Serial(ref msg)) if msg == "mock source exhausted" => break,
            Err(e) => {
                warn!(error = %e, "serial read error — skipping");
                continue;
            }
        };

        match serde_json::from_str::<Reading>(&line) {
            Ok(reading) => {
                info!(
                    target = reading.target,
                    average = reading.average,
                    action = %reading.action,
                    "reading ingested"
                );
                {
                    let mut lock = latest_reading.lock().unwrap();
                    *lock = Some(reading.clone());
                }

                if let Err(e) = store.write_reading(brew_id, &reading).await {
                    warn!(error = %e, brew_id, "failed to persist reading — continuing");
                }
            }
            Err(e) => {
                warn!(error = %e, raw_line = %line.trim(), "malformed reading skipped");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial::mock::MockSerialSource;
    use crate::store::fake::FakeTimeStore;

    const BREW_ID: &str = "test-brew";

    fn valid_line(target: f64, average: f64) -> String {
        format!(
            r#"{{"target":{target},"average":{average},"min":18.0,"max":19.0,"ambient":20.0,"action":"idle"}}"#
        )
    }

    #[tokio::test]
    async fn valid_readings_update_latest_state() {
        let lines = vec![Ok(valid_line(19.5, 18.2)), Ok(valid_line(19.5, 19.0))];
        let mut source = MockSerialSource::new(lines);
        let latest = Arc::new(Mutex::new(None));
        let store = FakeTimeStore::new();

        ingest_loop(&mut source, Arc::clone(&latest), &store, BREW_ID).await;

        let reading = latest
            .lock()
            .unwrap()
            .clone()
            .expect("should have a reading");
        assert_eq!(reading.average, 19.0); // last reading wins
        assert_eq!(reading.target, 19.5);
    }

    #[tokio::test]
    async fn malformed_lines_are_skipped_loop_continues() {
        let lines = vec![
            Ok(valid_line(19.5, 18.0)),
            Ok("not valid json".to_string()),
            Ok(valid_line(19.5, 19.5)),
        ];
        let mut source = MockSerialSource::new(lines);
        let latest = Arc::new(Mutex::new(None));
        let store = FakeTimeStore::new();

        ingest_loop(&mut source, Arc::clone(&latest), &store, BREW_ID).await;

        // Loop completed — malformed line did not crash it
        let reading = latest
            .lock()
            .unwrap()
            .clone()
            .expect("should have a reading");
        assert_eq!(reading.average, 19.5); // last valid reading
    }

    #[tokio::test]
    async fn serial_error_is_skipped_loop_continues() {
        let lines = vec![
            Ok(valid_line(19.5, 18.0)),
            Err("transient error".to_string()),
            Ok(valid_line(19.5, 19.2)),
        ];
        let mut source = MockSerialSource::new(lines);
        let latest = Arc::new(Mutex::new(None));
        let store = FakeTimeStore::new();

        ingest_loop(&mut source, Arc::clone(&latest), &store, BREW_ID).await;

        let reading = latest
            .lock()
            .unwrap()
            .clone()
            .expect("should have a reading");
        assert_eq!(reading.average, 19.2);
    }

    #[tokio::test]
    async fn valid_readings_are_written_to_the_store_tagged_by_brew_id() {
        let lines = vec![Ok(valid_line(19.5, 18.2)), Ok(valid_line(19.5, 19.0))];
        let mut source = MockSerialSource::new(lines);
        let latest = Arc::new(Mutex::new(None));
        let store = FakeTimeStore::new();

        ingest_loop(&mut source, Arc::clone(&latest), &store, BREW_ID).await;

        let persisted = store
            .last_reading(BREW_ID)
            .await
            .unwrap()
            .expect("reading should have been persisted");
        assert_eq!(persisted.average, 19.0); // last write wins
    }

    /// A `TimeStore` whose `write_reading` always fails, used to verify the
    /// ingest loop treats persistence failures as non-fatal.
    struct FailingTimeStore;

    #[async_trait::async_trait]
    impl TimeStore for FailingTimeStore {
        async fn write_reading(
            &self,
            _brew_id: &str,
            _reading: &Reading,
        ) -> crate::error::Result<()> {
            Err(AppError::Store("simulated write failure".to_string()))
        }

        async fn last_reading(&self, _brew_id: &str) -> crate::error::Result<Option<Reading>> {
            Ok(None)
        }

        async fn save_state(
            &self,
            _state: &crate::model::ControllerState,
        ) -> crate::error::Result<()> {
            Ok(())
        }

        async fn load_state(&self) -> crate::error::Result<Option<crate::model::ControllerState>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn rehydrates_from_persisted_reading_when_present() {
        let store = FakeTimeStore::new();
        store.seed_reading(
            BREW_ID,
            Reading {
                target: 19.5,
                average: 18.7,
                min: 18.0,
                max: 19.0,
                ambient: 20.0,
                action: "idle".to_string(),
                reason_code: String::new(),
                json_size: None,
            },
        );

        let rehydrated = rehydrate_latest_reading(&store, BREW_ID).await;

        assert_eq!(
            rehydrated
                .expect("should rehydrate persisted reading")
                .average,
            18.7
        );
    }

    #[tokio::test]
    async fn starts_empty_when_nothing_persisted() {
        let store = FakeTimeStore::new();

        let rehydrated = rehydrate_latest_reading(&store, BREW_ID).await;

        assert!(rehydrated.is_none());
    }

    #[tokio::test]
    async fn store_write_failure_is_logged_and_loop_continues() {
        let lines = vec![Ok(valid_line(19.5, 18.0)), Ok(valid_line(19.5, 19.2))];
        let mut source = MockSerialSource::new(lines);
        let latest = Arc::new(Mutex::new(None));
        let store = FailingTimeStore;

        ingest_loop(&mut source, Arc::clone(&latest), &store, BREW_ID).await;

        // In-memory state still updates even though every persistence write
        // failed — the loop must not crash or stop early.
        let reading = latest
            .lock()
            .unwrap()
            .clone()
            .expect("should have a reading");
        assert_eq!(reading.average, 19.2);
    }
}
