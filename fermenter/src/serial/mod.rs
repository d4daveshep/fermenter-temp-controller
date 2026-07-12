use async_trait::async_trait;

use crate::error::Result;

pub mod arduino;
pub mod mock;

/// Abstraction over the serial transport, so the ingest loop and web layer
/// never depend on whether they're driven by a scripted `MockSerialSource`
/// (hardware-free dev/test) or the real `arduino::ArduinoSerialSource`
/// (`tokio-serial` against physical hardware).
#[async_trait]
pub trait SerialSource: Send {
    /// Read one complete newline-terminated line from the source.
    async fn read_line(&mut self) -> Result<String>;

    /// Write a target temperature to the device using `<value>` framing,
    /// e.g. `<19.5>`.
    async fn write_target(&mut self, target: f64) -> Result<()>;
}
