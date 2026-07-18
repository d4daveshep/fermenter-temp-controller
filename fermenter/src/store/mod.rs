use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};

use crate::error::Result;
use crate::model::{ControllerState, Reading};

#[cfg(test)]
pub mod fake;
pub mod redis;

/// A complete, timestamped temperature observation for a brew.
#[derive(Clone, Debug, PartialEq)]
pub struct TemperatureSample {
    pub timestamp: DateTime<Utc>,
    pub fermenter: f64,
    pub ambient: f64,
    pub target: f64,
}

/// Abstraction over time-series persistence, enabling a fake store to drive
/// ingest-loop and rehydration tests without Redis or Docker.
#[async_trait]
pub trait TimeStore: Send + Sync {
    /// Persist `reading`, tagged by `brew_id`.
    async fn write_reading(&self, brew_id: &str, reading: &Reading) -> Result<()>;

    /// Fetch the most recently persisted reading for `brew_id`, if any.
    async fn last_reading(&self, brew_id: &str) -> Result<Option<Reading>>;

    /// Fetch complete temperature samples for `brew_id` in the inclusive time
    /// range, aggregated into timestamp-aligned buckets.
    async fn temperature_history(
        &self,
        brew_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        bucket: Duration,
    ) -> Result<Vec<TemperatureSample>>;

    /// Persist the current controller state (target temp + brew id).
    async fn save_state(&self, state: &ControllerState) -> Result<()>;

    /// Load the most recently persisted controller state, if any.
    async fn load_state(&self) -> Result<Option<ControllerState>>;
}
