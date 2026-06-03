use std::sync::{Arc, Mutex};

use tracing::{info, warn};

use crate::error::AppError;
use crate::model::Reading;
use crate::serial::SerialSource;

/// Ingest loop: reads lines from `source`, parses them as `Reading` values,
/// updates `latest_reading` on success, logs a warning and continues on
/// failure. Runs until the source is exhausted or returns a terminal error.
pub async fn ingest_loop(
    source: &mut dyn SerialSource,
    latest_reading: Arc<Mutex<Option<Reading>>>,
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
                let mut lock = latest_reading.lock().unwrap();
                *lock = Some(reading);
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

        ingest_loop(&mut source, Arc::clone(&latest)).await;

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

        ingest_loop(&mut source, Arc::clone(&latest)).await;

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

        ingest_loop(&mut source, Arc::clone(&latest)).await;

        let reading = latest
            .lock()
            .unwrap()
            .clone()
            .expect("should have a reading");
        assert_eq!(reading.average, 19.2);
    }
}
