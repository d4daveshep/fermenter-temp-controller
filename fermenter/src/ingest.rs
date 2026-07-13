use std::sync::{Arc, Mutex};

use tokio::sync::watch;
use tracing::{info, warn};

use crate::error::AppError;
use crate::model::Reading;
use crate::serial::SerialSource;
use crate::store::TimeStore;
use crate::temperature_control::target_to_write;

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
/// tagged with the brew identifier currently held by `brew_rx` — read once
/// per reading via `.borrow().clone()` and dropped before any `.await`
/// (design.md decision 3) — so a runtime brew switch only affects readings
/// persisted after it (brew-session: "Tag subsequent readings with the
/// newly active brew identifier"); a persistence failure is logged and does
/// not stop the loop (reading-history: "Persist readings..." scenario).
/// After each successfully parsed reading, the current desired target
/// (`target_rx`) is compared against the reading's self-reported target and
/// written to the device if they differ (temperature-control: "Reconcile
/// the target temperature to the device" — reconcile piggybacks on the
/// reading cadence rather than a separate concurrent listener; see
/// design.md decision 2). Runs until the source is exhausted or returns a
/// terminal error.
pub async fn ingest_loop(
    source: &mut dyn SerialSource,
    latest_reading: Arc<Mutex<Option<Reading>>>,
    store: &dyn TimeStore,
    brew_rx: &watch::Receiver<String>,
    target_rx: &watch::Receiver<f64>,
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

                let brew_id = brew_rx.borrow().clone();
                if let Err(e) = store.write_reading(&brew_id, &reading).await {
                    warn!(error = %e, brew_id, "failed to persist reading — continuing");
                }

                let desired = *target_rx.borrow();
                if let Some(to_write) = target_to_write(desired, reading.target) {
                    info!(
                        desired = to_write,
                        reported = reading.target,
                        "target temperature needs updating — writing to device"
                    );
                    if let Err(e) = source.write_target(to_write).await {
                        warn!(error = %e, "failed to write target to device — continuing");
                    }
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

    /// A `watch::Receiver` whose value is irrelevant to the test at hand —
    /// used by tests that predate target reconcile and don't assert on
    /// `written_targets`.
    fn no_op_target_rx() -> watch::Receiver<f64> {
        watch::channel(19.5).1
    }

    /// A `watch::Receiver<String>` fixed at `id` for the duration of the
    /// test — used by tests that predate dynamic brew tagging and don't
    /// exercise a runtime brew-identifier change.
    fn fixed_brew_rx(id: &str) -> watch::Receiver<String> {
        watch::channel(id.to_string()).1
    }

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

        ingest_loop(
            &mut source,
            Arc::clone(&latest),
            &store,
            &fixed_brew_rx(BREW_ID),
            &no_op_target_rx(),
        )
        .await;

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

        ingest_loop(
            &mut source,
            Arc::clone(&latest),
            &store,
            &fixed_brew_rx(BREW_ID),
            &no_op_target_rx(),
        )
        .await;

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

        ingest_loop(
            &mut source,
            Arc::clone(&latest),
            &store,
            &fixed_brew_rx(BREW_ID),
            &no_op_target_rx(),
        )
        .await;

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

        ingest_loop(
            &mut source,
            Arc::clone(&latest),
            &store,
            &fixed_brew_rx(BREW_ID),
            &no_op_target_rx(),
        )
        .await;

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

        ingest_loop(
            &mut source,
            Arc::clone(&latest),
            &store,
            &fixed_brew_rx(BREW_ID),
            &no_op_target_rx(),
        )
        .await;

        // In-memory state still updates even though every persistence write
        // failed — the loop must not crash or stop early.
        let reading = latest
            .lock()
            .unwrap()
            .clone()
            .expect("should have a reading");
        assert_eq!(reading.average, 19.2);
    }

    #[tokio::test]
    async fn reconcile_writes_target_when_it_differs_from_reported() {
        // Reading reports the device's current target as 19.5; the desired
        // (watch) value is 20.0 — they differ, so a write is expected.
        let lines = vec![Ok(valid_line(19.5, 18.2))];
        let mut source = MockSerialSource::new(lines);
        let latest = Arc::new(Mutex::new(None));
        let store = FakeTimeStore::new();
        let (_tx, rx) = watch::channel(20.0);

        ingest_loop(
            &mut source,
            Arc::clone(&latest),
            &store,
            &fixed_brew_rx(BREW_ID),
            &rx,
        )
        .await;

        assert_eq!(source.written_targets, vec![20.0]);
    }

    #[tokio::test]
    async fn reconcile_skips_write_when_target_already_matches_reported() {
        // Reading reports the device's current target as 19.5; the desired
        // (watch) value is also 19.5 — already in sync, no write expected.
        let lines = vec![Ok(valid_line(19.5, 18.2))];
        let mut source = MockSerialSource::new(lines);
        let latest = Arc::new(Mutex::new(None));
        let store = FakeTimeStore::new();
        let (_tx, rx) = watch::channel(19.5);

        ingest_loop(
            &mut source,
            Arc::clone(&latest),
            &store,
            &fixed_brew_rx(BREW_ID),
            &rx,
        )
        .await;

        assert!(source.written_targets.is_empty());
    }

    #[tokio::test]
    async fn reconcile_reflects_the_desired_target_updated_between_reading_batches() {
        // Two readings both report the device's current target as 19.5.
        // While the desired value stays 19.5 (in sync), no write occurs.
        // Once the operator changes the desired value to 21.0 (simulating a
        // `POST /target` landing between readings), the next processed
        // reading reconciles against the new value.
        let latest = Arc::new(Mutex::new(None));
        let store = FakeTimeStore::new();
        let (tx, rx) = watch::channel(19.5);

        let mut first_batch = MockSerialSource::new(vec![Ok(valid_line(19.5, 18.2))]);
        ingest_loop(
            &mut first_batch,
            Arc::clone(&latest),
            &store,
            &fixed_brew_rx(BREW_ID),
            &rx,
        )
        .await;
        assert!(
            first_batch.written_targets.is_empty(),
            "no write expected before the target changes"
        );

        tx.send(21.0).unwrap();

        let mut second_batch = MockSerialSource::new(vec![Ok(valid_line(19.5, 18.5))]);
        ingest_loop(
            &mut second_batch,
            Arc::clone(&latest),
            &store,
            &fixed_brew_rx(BREW_ID),
            &rx,
        )
        .await;
        assert_eq!(
            second_batch.written_targets,
            vec![21.0],
            "write expected once the desired target changed"
        );
    }

    #[tokio::test]
    async fn valid_readings_are_written_to_the_store_under_the_watch_channels_brew_id() {
        // brew-session: "Tag subsequent readings with the newly active brew
        // identifier" — the ingest loop tags persisted readings with
        // whatever brew id the watch channel currently holds, not a fixed
        // parameter.
        let lines = vec![Ok(valid_line(19.5, 18.2))];
        let mut source = MockSerialSource::new(lines);
        let latest = Arc::new(Mutex::new(None));
        let store = FakeTimeStore::new();
        let brew_rx = fixed_brew_rx("dynamic-brew");

        ingest_loop(
            &mut source,
            Arc::clone(&latest),
            &store,
            &brew_rx,
            &no_op_target_rx(),
        )
        .await;

        let persisted = store
            .last_reading("dynamic-brew")
            .await
            .unwrap()
            .expect("reading should have been persisted under the watch channel's brew id");
        assert_eq!(persisted.average, 18.2);
    }

    #[tokio::test]
    async fn brew_switch_between_batches_only_affects_readings_processed_after_it() {
        // Mirrors reconcile_reflects_the_desired_target_updated_between_reading_batches:
        // a reading processed before the brew id changes is tagged with the
        // old id; a reading processed after the change is tagged with the
        // new one, and the old id's persisted data is unaffected.
        let latest = Arc::new(Mutex::new(None));
        let store = FakeTimeStore::new();
        let (brew_tx, brew_rx) = watch::channel("brew-a".to_string());

        let mut first_batch = MockSerialSource::new(vec![Ok(valid_line(19.5, 18.0))]);
        ingest_loop(
            &mut first_batch,
            Arc::clone(&latest),
            &store,
            &brew_rx,
            &no_op_target_rx(),
        )
        .await;

        brew_tx.send("brew-b".to_string()).unwrap();

        let mut second_batch = MockSerialSource::new(vec![Ok(valid_line(19.5, 19.0))]);
        ingest_loop(
            &mut second_batch,
            Arc::clone(&latest),
            &store,
            &brew_rx,
            &no_op_target_rx(),
        )
        .await;

        let brew_a = store
            .last_reading("brew-a")
            .await
            .unwrap()
            .expect("brew-a should still have its reading persisted");
        assert_eq!(brew_a.average, 18.0);

        let brew_b = store
            .last_reading("brew-b")
            .await
            .unwrap()
            .expect("brew-b should have the reading persisted after the switch");
        assert_eq!(brew_b.average, 19.0);
    }
}
