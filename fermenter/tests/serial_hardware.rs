//! Hardware-in-the-loop tests for `serial::arduino::ArduinoSerialSource`
//! against a real, physically attached Arduino running the current
//! `TempController` firmware (see `arduino/TempController/TempController.ino`).
//!
//! All tests here are `#[ignore]`'d: they never run under plain `cargo test`
//! (and never run in CI, which has no attached Arduino), only via
//! `cargo test -- --ignored` on a machine with the device plugged in.
//!
//! No temperature sensors are required. These tests only assert on the
//! serial *framing* contract (a newline-terminated JSON line was read; a
//! `<target>` write round-trips into a subsequent reading's `target`
//! field) — not on temperature plausibility. Without DallasTemperature
//! sensors attached, `instant`/`average`/`min`/`max`/`ambient` will read
//! around `-127.0` (the library's `DEVICE_DISCONNECTED_C` sentinel), which
//! is expected and irrelevant to what's being verified here.
//!
//! Configure the port via `SERIAL_PORT`/`SERIAL_BAUD` env vars (same names
//! `Config` reads); defaults to `/dev/ttyACM0` @ `115200` if unset, matching
//! `docs/rewrite-plan.md` §9's documented default.

use std::time::Duration;

use fermenter::model::Reading;
use fermenter::serial::SerialSource;
use fermenter::serial::arduino::ArduinoSerialSource;

/// The firmware prints a JSON reading roughly every 10s
/// (`TempController.ino`'s `printJsonEvery10Secs`). Opening the port also
/// toggles DTR on genuine Arduino Uno boards, which resets the sketch (its
/// own ~1-2s of `setup()` delays run again before the 10s print cycle
/// starts), so the first line after any (re)connect can take ~12-13s to
/// arrive. The timeout is generous to avoid flaking on that reset, while
/// still failing fast if the device truly isn't producing output.
const READ_TIMEOUT: Duration = Duration::from_secs(20);
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

#[tokio::test]
#[ignore]
async fn opens_real_port() {
    let mut source = ArduinoSerialSource::new(test_port(), test_baud());

    // The port opens lazily on first use; a successful read implies the
    // open succeeded (this is the only externally observable signal the
    // `SerialSource` trait exposes — there's no separate "is connected"
    // check by design, matching the mock's interface).
    let outcome = tokio::time::timeout(READ_TIMEOUT, source.read_line()).await;

    assert!(
        outcome.is_ok(),
        "timed out waiting for the serial port at {} to open and produce a line — \
         is the Arduino plugged in and running TempController firmware?",
        test_port()
    );
    assert!(
        outcome.unwrap().is_ok(),
        "expected the port to open and read successfully"
    );
}

#[tokio::test]
#[ignore]
async fn reads_a_real_line() {
    let mut source = ArduinoSerialSource::new(test_port(), test_baud());

    let line = tokio::time::timeout(READ_TIMEOUT, source.read_line())
        .await
        .expect("timed out waiting for a line from the device")
        .expect("read_line should succeed");

    assert!(!line.is_empty(), "expected a non-empty line");
    assert!(
        !line.contains('\n'),
        "read_line should strip the trailing newline, matching the mock's contract"
    );

    // Don't assert on field values — no sensors attached, so
    // instant/average/min/max/ambient may read ~-127 (DEVICE_DISCONNECTED_C).
    // Only the shape of the contract matters here.
    let reading: Reading =
        serde_json::from_str(&line).expect("line should parse as a well-formed Reading");
    assert!(!reading.action.is_empty(), "action should be present");
}

#[tokio::test]
#[ignore]
async fn write_target_roundtrips() {
    let mut source = ArduinoSerialSource::new(test_port(), test_baud());

    // A value distinct enough from common defaults (20.0) to make the
    // round-trip observable; nonzero, since the firmware's
    // `getUpdatedTargetTemp` treats a parsed `0.0` as "no update".
    let target = 21.3;

    source
        .write_target(target)
        .await
        .expect("write_target should succeed");

    let deadline = tokio::time::Instant::now() + ROUNDTRIP_TIMEOUT;
    let mut observed_target = None;

    while tokio::time::Instant::now() < deadline {
        let Ok(Ok(line)) = tokio::time::timeout(READ_TIMEOUT, source.read_line()).await else {
            continue;
        };
        if let Ok(reading) = serde_json::from_str::<Reading>(&line)
            && (reading.target - target).abs() < 0.01
        {
            observed_target = Some(reading.target);
            break;
        }
    }

    assert_eq!(
        observed_target,
        Some(target),
        "expected a subsequent reading to report target == {target} within {:?} of writing it",
        ROUNDTRIP_TIMEOUT
    );
}
