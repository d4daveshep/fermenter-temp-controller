use std::time::Duration;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tracing::{info, warn};

use crate::error::{AppError, Result};
use crate::serial::SerialSource;

/// Initial reconnect delay. Doubled on each consecutive failure, capped at
/// `MAX_BACKOFF` (device-connection: "Serial connection resilience").
const INITIAL_BACKOFF: Duration = Duration::from_millis(500);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Real serial transport, backed by `tokio-serial`, satisfying the same
/// `SerialSource` contract as `MockSerialSource` so no dependent code
/// (ingest/state/web) changes when this replaces the mock.
///
/// The port is opened lazily (on first `read_line`/`write_target` call) and
/// reopened automatically ONLY if the stream reaches EOF (the device was
/// physically disconnected). Transient read, write, and flush errors do NOT
/// close the stream — the error is returned to the caller, and the next call
/// retries on the same open connection. This avoids reopening the port on
/// transient USB serial noise, which on a genuine Arduino would trigger a
/// DTR-based hardware reset (resetting the firmware's lifetime min/max
/// temperature tracking).
///
/// On Linux the kernel asserts DTR as a side effect of the `open()` syscall
/// (a USB CDC ACM protocol requirement), so the very first open at startup
/// always resets the Arduino. The `dtr_on_open(false)` builder setting
/// clears DTR immediately after open and prevents re-assertion on platforms
/// where the `serialport` crate can control it, but the kernel-level pulse
/// during `open()` itself is unavoidable. The reconnection behaviour
/// ("only on EOF, not on transient errors") is what prevents subsequent
/// resets during normal operation.
///
/// Every failure to open the port sleeps for the current backoff delay
/// (doubling each consecutive failure, capped at `MAX_BACKOFF`) before
/// returning an error, so a caller that loops on error without its own delay
/// (as `ingest_loop` does) still retries at a bounded rate rather than busy
/// looping. A successful open or read resets the backoff to its initial
/// value.
pub struct ArduinoSerialSource {
    port_path: String,
    baud_rate: u32,
    stream: Option<BufReader<SerialStream>>,
    backoff: Duration,
}

impl ArduinoSerialSource {
    /// Create a source for the given port path (e.g. `/dev/ttyACM0`) and
    /// baud rate (115200 to match the fixed firmware contract). The port is
    /// not opened until the first read or write.
    pub fn new(port_path: impl Into<String>, baud_rate: u32) -> Self {
        Self {
            port_path: port_path.into(),
            baud_rate,
            stream: None,
            backoff: INITIAL_BACKOFF,
        }
    }

    fn bump_backoff(&mut self) {
        self.backoff = std::cmp::min(self.backoff * 2, MAX_BACKOFF);
    }

    fn reset_backoff(&mut self) {
        self.backoff = INITIAL_BACKOFF;
    }

    /// Ensure the port is open, opening it (and sleeping for the current
    /// backoff on failure) if it is not already.
    async fn ensure_connected(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        match tokio_serial::new(&self.port_path, self.baud_rate)
            // On Linux the kernel asserts DTR during the `open()` syscall
            // regardless, but `dtr_on_open(false)` clears DTR immediately
            // after and helps on platforms where the `serialport` crate has
            // control over it at open time (e.g. Windows, macOS). See the
            // struct-level doc for the full reasoning.
            .dtr_on_open(false)
            .open_native_async()
        {
            Ok(port) => {
                info!(port = %self.port_path, baud = self.baud_rate, "serial port opened");
                self.stream = Some(BufReader::new(port));
                self.reset_backoff();
                Ok(())
            }
            Err(e) => {
                warn!(
                    error = %e,
                    port = %self.port_path,
                    backoff_ms = self.backoff.as_millis(),
                    "failed to open serial port — retrying after backoff"
                );
                tokio::time::sleep(self.backoff).await;
                self.bump_backoff();
                Err(AppError::Serial(format!(
                    "failed to open {}: {e}",
                    self.port_path
                )))
            }
        }
    }
}

#[async_trait]
impl SerialSource for ArduinoSerialSource {
    async fn read_line(&mut self) -> Result<String> {
        self.ensure_connected().await?;
        let stream = self
            .stream
            .as_mut()
            .expect("ensure_connected returned Ok, so stream is Some");

        let mut line = String::new();
        match stream.read_line(&mut line).await {
            Ok(0) => {
                // EOF: the device end of the stream closed (e.g. unplugged).
                self.stream = None;
                warn!(
                    port = %self.port_path,
                    backoff_ms = self.backoff.as_millis(),
                    "serial port closed (EOF) — reconnecting after backoff"
                );
                tokio::time::sleep(self.backoff).await;
                self.bump_backoff();
                Err(AppError::Serial(format!(
                    "serial port {} closed (EOF)",
                    self.port_path
                )))
            }
            Ok(_) => {
                self.reset_backoff();
                let trimmed = line.trim_end_matches(['\n', '\r']).to_string();
                Ok(trimmed)
            }
            Err(e) => {
                // Transient read error (e.g. USB noise, buffer overrun).
                // Keep the stream alive — the next call retries on the same
                // connection rather than reopening (which would DTR-reset a
                // genuine Arduino, resetting its lifetime min/max tracking).
                warn!(
                    error = %e,
                    port = %self.port_path,
                    "serial read error — retrying on same connection"
                );
                Err(AppError::Serial(format!(
                    "serial read error on {}: {e}",
                    self.port_path
                )))
            }
        }
    }

    async fn write_target(&mut self, target: f64) -> Result<()> {
        self.ensure_connected().await?;
        let stream = self
            .stream
            .as_mut()
            .expect("ensure_connected returned Ok, so stream is Some");

        // `<value>` framing, matching the fixed firmware contract
        // (device-connection: "Serial framing and baud contract").
        let framed = format!("<{target}>");
        if let Err(e) = stream.get_mut().write_all(framed.as_bytes()).await {
            // Transient write error. Keep the stream alive to avoid
            // DTR-resetting the Arduino on reopen.
            warn!(
                error = %e,
                port = %self.port_path,
                "serial write error — retrying on same connection"
            );
            return Err(AppError::Serial(format!(
                "serial write error on {}: {e}",
                self.port_path
            )));
        }
        if let Err(e) = stream.get_mut().flush().await {
            // Transient flush error. Keep the stream alive.
            warn!(
                error = %e,
                port = %self.port_path,
                "serial flush error — retrying on same connection"
            );
            return Err(AppError::Serial(format!(
                "serial flush error on {}: {e}",
                self.port_path
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic, hardware-free proof of the backoff-doubling mechanism
    /// (device-connection: "Serial connection resilience"). Points at a
    /// path that can never successfully open, so every `read_line` call
    /// fails at `ensure_connected` and we can assert the sleep durations
    /// double each time, capped at `MAX_BACKOFF` — without needing any
    /// physical device attached or unplugged. Runs on tokio's paused
    /// virtual clock (`start_paused = true`) so it completes instantly
    /// instead of actually sleeping through several backoff cycles up to
    /// the 30s cap. The genuinely hardware-in-the-loop unplug/replug
    /// scenario (task 4.2) can't be automated this way and remains a
    /// manual verification step (see `tests/serial_hardware.rs` doc comment
    /// and `openspec/changes/slice-6-real-hardware/tasks.md`).
    #[tokio::test(start_paused = true)]
    async fn backoff_doubles_on_repeated_open_failure_then_caps() {
        let mut source = ArduinoSerialSource::new("/dev/does-not-exist-for-testing", 115_200);

        let mut elapsed_per_attempt = Vec::new();
        for _ in 0..8 {
            let start = tokio::time::Instant::now();
            let result = source.read_line().await;
            elapsed_per_attempt.push(start.elapsed());
            assert!(result.is_err(), "opening a nonexistent port must fail");
        }

        assert_eq!(elapsed_per_attempt[0], INITIAL_BACKOFF);
        assert_eq!(elapsed_per_attempt[1], INITIAL_BACKOFF * 2);
        assert_eq!(elapsed_per_attempt[2], INITIAL_BACKOFF * 4);
        assert_eq!(elapsed_per_attempt[3], INITIAL_BACKOFF * 8);

        // Keeps doubling until it hits MAX_BACKOFF, then stays capped there
        // rather than growing unbounded.
        let mut expected = INITIAL_BACKOFF;
        for actual in &elapsed_per_attempt {
            assert_eq!(*actual, expected);
            expected = std::cmp::min(expected * 2, MAX_BACKOFF);
        }
    }
}
