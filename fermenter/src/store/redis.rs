use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use std::collections::HashMap;

use crate::error::{AppError, Result};
use crate::model::{ControllerState, Reading};
use crate::store::{TemperatureSample, TimeStore};

/// Redis key holding the persisted controller state hash.
const STATE_KEY: &str = "controller:state";

/// Fields (series suffix, value) persisted per reading, matching
/// `rewrite-plan.md` §8's `temp:{brew_id}:{field}` schema.
fn series_fields(reading: &Reading) -> [(&'static str, f64); 3] {
    [
        ("fermenter", reading.average),
        ("ambient", reading.ambient),
        ("target", reading.target),
    ]
}

fn series_key(brew_id: &str, field: &str) -> String {
    format!("temp:{brew_id}:{field}")
}

fn latest_reading_key(brew_id: &str) -> String {
    format!("reading:{brew_id}:latest")
}

fn store_err(e: impl std::fmt::Display) -> AppError {
    AppError::Store(e.to_string())
}

/// `TimeStore` implementation backed by Redis 8 Time Series (built in).
///
/// Uses a `ConnectionManager`, which transparently reconnects on transient
/// connection errors (design.md decision 5) — no explicit reconnect loop is
/// needed here.
pub struct RedisTimeStore {
    manager: ConnectionManager,
    retention_ms: i64,
}

impl RedisTimeStore {
    /// Build a store for `redis_url`, configuring newly-created series with
    /// `retention_days` of retention.
    ///
    /// The underlying connection is established lazily (on first use) rather
    /// than eagerly here, so a Redis outage at startup does not block the
    /// application from starting — only config *values* are fail-fast
    /// (design.md decision 5). Once connected, the `ConnectionManager`
    /// reconnects automatically on transient errors.
    pub fn connect(redis_url: &str, retention_days: u32) -> Result<Self> {
        let client = redis::Client::open(redis_url).map_err(store_err)?;
        let manager = client
            .get_connection_manager_lazy(redis::aio::ConnectionManagerConfig::new())
            .map_err(store_err)?;
        let retention_ms = i64::from(retention_days) * 24 * 60 * 60 * 1000;
        Ok(Self {
            manager,
            retention_ms,
        })
    }

    /// Create `key` as a time series with the configured retention and
    /// `brew`/`field` labels, tolerating an already-existing series.
    async fn ensure_series(&self, key: &str, brew_id: &str, field: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        let result: std::result::Result<(), redis::RedisError> = redis::cmd("TS.CREATE")
            .arg(key)
            .arg("RETENTION")
            .arg(self.retention_ms)
            .arg("LABELS")
            .arg("brew")
            .arg(brew_id)
            .arg("field")
            .arg(field)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(()) => Ok(()),
            Err(e) if e.to_string().contains("already exists") => Ok(()),
            Err(e) => Err(store_err(e)),
        }
    }
}

#[async_trait]
impl TimeStore for RedisTimeStore {
    async fn write_reading(&self, brew_id: &str, reading: &Reading) -> Result<()> {
        for (field, value) in series_fields(reading) {
            let key = series_key(brew_id, field);
            self.ensure_series(&key, brew_id, field).await?;

            let mut conn = self.manager.clone();
            redis::cmd("TS.ADD")
                .arg(&key)
                .arg("*")
                .arg(value)
                .query_async::<i64>(&mut conn)
                .await
                .map_err(store_err)?;
        }

        // Full-fidelity cache for rehydration (design.md decision 3a) — the
        // per-field series alone can't losslessly reconstruct min/max/action.
        let mut conn = self.manager.clone();
        let json = serde_json::to_string(reading)?;
        conn.set::<_, _, ()>(latest_reading_key(brew_id), json)
            .await
            .map_err(store_err)?;

        Ok(())
    }

    async fn last_reading(&self, brew_id: &str) -> Result<Option<Reading>> {
        let mut conn = self.manager.clone();
        let raw: Option<String> = conn
            .get(latest_reading_key(brew_id))
            .await
            .map_err(store_err)?;

        match raw {
            Some(json) => Ok(Some(serde_json::from_str(&json)?)),
            None => Ok(None),
        }
    }

    async fn temperature_history(
        &self,
        brew_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        bucket: Duration,
    ) -> Result<Vec<TemperatureSample>> {
        let bucket_ms = bucket.num_milliseconds();
        if bucket_ms <= 0 {
            return Err(AppError::Store(
                "temperature history bucket must be positive".to_string(),
            ));
        }

        let start_ms = start.timestamp_millis();
        let end_ms = end.timestamp_millis();
        let mut series = HashMap::new();
        for field in ["fermenter", "ambient", "target"] {
            let mut conn = self.manager.clone();
            let values: std::result::Result<Vec<(i64, f64)>, redis::RedisError> =
                redis::cmd("TS.RANGE")
                    .arg(series_key(brew_id, field))
                    .arg(start_ms)
                    .arg(end_ms)
                    .arg("ALIGN")
                    // Anchor buckets to the epoch so polling a rolling window
                    // cannot shift sample boundaries between requests.
                    .arg(0)
                    .arg("AGGREGATION")
                    .arg("avg")
                    .arg(bucket_ms)
                    .query_async(&mut conn)
                    .await;
            let values = match values {
                Ok(values) => values,
                Err(error) if error.to_string().contains("key does not exist") => {
                    return Ok(Vec::new());
                }
                Err(error) => return Err(store_err(error)),
            };
            series.insert(field, values.into_iter().collect::<HashMap<_, _>>());
        }

        let fermenter = &series["fermenter"];
        let ambient = &series["ambient"];
        let target = &series["target"];
        let mut samples: Vec<_> = fermenter
            .iter()
            .filter_map(|(timestamp, fermenter)| {
                Some(TemperatureSample {
                    timestamp: Utc.timestamp_millis_opt(*timestamp).single()?,
                    fermenter: *fermenter,
                    ambient: *ambient.get(timestamp)?,
                    target: *target.get(timestamp)?,
                })
            })
            .collect();
        samples.sort_by_key(|sample| sample.timestamp);
        Ok(samples)
    }

    async fn save_state(&self, state: &ControllerState) -> Result<()> {
        let mut conn = self.manager.clone();
        conn.hset_multiple::<_, _, _, ()>(
            STATE_KEY,
            &[
                ("target_temp", state.target_temp.to_string()),
                ("brew_id", state.brew_id.clone()),
            ],
        )
        .await
        .map_err(store_err)?;
        Ok(())
    }

    async fn load_state(&self) -> Result<Option<ControllerState>> {
        let mut conn = self.manager.clone();
        let target_temp: Option<String> = conn
            .hget(STATE_KEY, "target_temp")
            .await
            .map_err(store_err)?;
        let brew_id: Option<String> = conn.hget(STATE_KEY, "brew_id").await.map_err(store_err)?;

        match (target_temp, brew_id) {
            (Some(target_temp), Some(brew_id)) => {
                let target_temp = target_temp.parse::<f64>().map_err(store_err)?;
                Ok(Some(ControllerState {
                    target_temp,
                    brew_id,
                }))
            }
            _ => Ok(None),
        }
    }
}
