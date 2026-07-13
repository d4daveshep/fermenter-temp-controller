use async_trait::async_trait;

use crate::error::Result;
use crate::model::{ControllerState, Reading};

#[cfg(test)]
pub mod fake;
pub mod redis;

/// Abstraction over time-series persistence, enabling a fake store to drive
/// ingest-loop and rehydration tests without Redis or Docker.
#[async_trait]
pub trait TimeStore: Send + Sync {
    /// Persist `reading`, tagged by `brew_id`.
    async fn write_reading(&self, brew_id: &str, reading: &Reading) -> Result<()>;

    /// Fetch the most recently persisted reading for `brew_id`, if any.
    async fn last_reading(&self, brew_id: &str) -> Result<Option<Reading>>;

    /// Persist the current controller state (target temp + brew id).
    async fn save_state(&self, state: &ControllerState) -> Result<()>;

    /// Load the most recently persisted controller state, if any.
    async fn load_state(&self) -> Result<Option<ControllerState>>;
}
