use async_trait::async_trait;

use crate::error::Result;

pub mod mock;

// NOTE: The real tokio-serial implementation is deferred to slice-6.
// The reconnect/backoff requirement is specified in the device-connection spec
// and will be implemented alongside the real transport.

/// Abstraction over the serial transport, enabling the mock source to drive
/// the ingest loop in place of real hardware.
#[async_trait]
pub trait SerialSource: Send {
    /// Read one complete newline-terminated line from the source.
    async fn read_line(&mut self) -> Result<String>;

    /// Write a target temperature to the device using `<value>` framing,
    /// e.g. `<19.5>`. Unused in main until slice-4 but part of the contract now.
    #[allow(dead_code)]
    async fn write_target(&mut self, target: f64) -> Result<()>;
}
