//! Hardware-in-the-loop test for the ESP32-C3 firmware's JSON telemetry
//! output (tasks.md task 17.3), against a real, physically attached board
//! running the current `firmware/esp32c3` binary.
//!
//! `#[ignore]`'d: never runs under plain `cargo test` (or in CI, which has
//! no attached board), only via `cargo test -- --ignored` on a machine with
//! the device plugged in. Lives in `firmware/logic` (not `firmware/esp32c3`)
//! because it needs a real, host-side serial port — `esp32c3`'s
//! `.cargo/config.toml` pins every cargo invocation there to the
//! `riscv32imc-unknown-none-elf` embedded target, and its own lib links
//! `esp-hal`/`esp-rtos`/`onecable`, none of which build for a host target.
//! `logic` already runs its tests on the host with no embedded toolchain
//! (`#![cfg_attr(not(test), no_std)]` in `lib.rs` makes it std-enabled
//! under `cargo test`), so it can host this test with no target override.
//!
//! Mirrors the shape of `fermenter/tests/serial_hardware.rs`'s
//! `reads_a_real_line` test (same retry-until-timeout tolerance for
//! boot-reset noise — see design.md Decision 7 / `firmware/README.md`: the
//! ESP32-C3 resets when the port opens, and its bootloader log lines don't
//! parse as JSON). Uses a local `Reading` mirror rather than depending on
//! the `fermenter` crate directly — `firmware/` is an independent
//! workspace (see repo root `CLAUDE.md`) and this test only needs to
//! confirm the wire format, not exercise the host crate itself.

use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};

use serde::Deserialize;

/// Mirrors `fermenter::model::Reading`'s shape (see design.md Decision 5 —
/// "Fields emitted"). Kept as a plain owned-`String` struct since it's
/// deserializing a runtime-read line, unlike `logic::TelemetryData`'s
/// `&'static str` fields, which only support serialising fixed strings.
#[derive(Debug, Deserialize)]
struct Reading {
    #[allow(dead_code)]
    target: f64,
    #[allow(dead_code)]
    average: f64,
    #[allow(dead_code)]
    min: f64,
    #[allow(dead_code)]
    max: f64,
    #[allow(dead_code)]
    ambient: f64,
    action: String,
    #[serde(rename = "reason-code")]
    reason_code: String,
}

/// The firmware emits a JSON line roughly every 10-16s (see tasks.md task
/// 17.1's soak-test note on why it's not exactly 10s) — generous enough to
/// also cover the initial boot-reset noise burst.
const ROUNDTRIP_TIMEOUT: Duration = Duration::from_secs(60);

fn test_port() -> String {
    std::env::var("SERIAL_PORT").unwrap_or_else(|_| "/dev/ttyACM0".to_string())
}

fn test_baud() -> u32 {
    std::env::var("SERIAL_BAUD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(115_200)
}

#[test]
#[ignore]
fn emits_a_valid_reading_line() {
    let port = serialport::new(test_port(), test_baud())
        .timeout(Duration::from_secs(20))
        .open()
        .unwrap_or_else(|e| {
            panic!(
                "failed to open serial port {} - is the board attached? ({e})",
                test_port()
            )
        });
    let mut reader = BufReader::new(port);

    let deadline = Instant::now() + ROUNDTRIP_TIMEOUT;
    let mut parsed: Option<Reading> = None;
    let mut last_line = String::new();

    while Instant::now() < deadline {
        let mut line = String::new();
        // A per-read timeout on the port itself bounds each `read_line`
        // call, so a silent board fails this with a read error well before
        // `ROUNDTRIP_TIMEOUT` - not an unbounded hang.
        if reader.read_line(&mut line).is_err() {
            continue;
        }
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        last_line = line.to_string();
        if let Ok(reading) = serde_json::from_str::<Reading>(line) {
            parsed = Some(reading);
            break;
        }
    }

    let reading = parsed.unwrap_or_else(|| {
        panic!(
            "no line parsed as a well-formed Reading within {ROUNDTRIP_TIMEOUT:?}; \
             last line seen: {last_line:?}"
        )
    });
    assert!(!reading.action.is_empty(), "action should be present");
    assert!(
        !reading.reason_code.is_empty(),
        "reason-code should be present"
    );
}
