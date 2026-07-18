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
use tokio::sync::Mutex;

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

/// All three tests in this file open the *same physical* serial device.
/// `serialport` (via `tokio-serial`) opens ports in exclusive mode by
/// default (`TIOCEXCL` + `flock`), and `cargo test` runs tests across
/// multiple threads concurrently unless `--test-threads=1` is passed. Without
/// this guard, two tests racing for the same exclusive-lock device produces
/// exactly the flaky failures this was built to prevent: the loser's
/// `ensure_connected()` fails "busy", backs off, and by the time it
/// acquires the lock the Arduino has often *just* been DTR-reset by
/// whichever test grabbed it last — so the first bytes read can be
/// boot-time noise/a truncated fragment rather than a clean JSON line,
/// causing spurious parse failures or an apparently-never-observed target
/// (mirrors `config.rs`'s `env_lock()` pattern for the same class of
/// problem — serializing access to a single shared external resource —
/// but uses a `tokio::sync::Mutex` rather than `std::sync::Mutex` since the
/// guard needs to stay held across `.await` points for a test's whole
/// duration; clippy's `await_holding_lock` correctly flags a std mutex used
/// this way).
static HARDWARE_LOCK: Mutex<()> = Mutex::const_new(());

#[tokio::test]
#[ignore]
async fn opens_real_port() {
    let _guard = HARDWARE_LOCK.lock().await;
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
    let _guard = HARDWARE_LOCK.lock().await;
    let mut source = ArduinoSerialSource::new(test_port(), test_baud());

    // The very first line read right after a fresh connection can
    // occasionally be truncated/corrupted — the same real-world artifact
    // `ingest_loop` is deliberately built to tolerate ("malformed reading
    // skipped", logged and continued, not fatal; observed directly during
    // slice-7's manual container verification). This test mirrors that
    // same tolerance: retry within the round-trip budget rather than
    // asserting the very first line read is always well-formed, which is a
    // stricter contract than production actually relies on.
    let deadline = tokio::time::Instant::now() + ROUNDTRIP_TIMEOUT;
    let mut parsed: Option<Reading> = None;
    let mut last_line = String::new();

    while tokio::time::Instant::now() < deadline {
        let Ok(Ok(line)) = tokio::time::timeout(READ_TIMEOUT, source.read_line()).await else {
            continue;
        };
        assert!(!line.is_empty(), "expected a non-empty line");
        assert!(
            !line.contains('\n'),
            "read_line should strip the trailing newline, matching the mock's contract"
        );
        last_line = line.clone();
        if let Ok(reading) = serde_json::from_str::<Reading>(&line) {
            parsed = Some(reading);
            break;
        }
    }

    // Don't assert on field values — no sensors attached, so
    // instant/average/min/max/ambient may read ~-127 (DEVICE_DISCONNECTED_C).
    // Only the shape of the contract matters here.
    let reading = parsed.unwrap_or_else(|| {
        panic!("no line parsed as a well-formed Reading within {ROUNDTRIP_TIMEOUT:?}; last line seen: {last_line:?}")
    });
    assert!(!reading.action.is_empty(), "action should be present");
}

#[tokio::test]
#[ignore]
async fn write_target_roundtrips() {
    let _guard = HARDWARE_LOCK.lock().await;
    let mut source = ArduinoSerialSource::new(test_port(), test_baud());

    // A value distinct enough from common defaults (20.0) to make the
    // round-trip observable; nonzero, since the firmware's
    // `getUpdatedTargetTemp` treats a parsed `0.0` as "no update".
    let target = 21.3;

    let deadline = tokio::time::Instant::now() + ROUNDTRIP_TIMEOUT;
    let mut observed_target = None;

    while tokio::time::Instant::now() < deadline {
        // Retry the write every iteration (i.e. roughly once per received
        // line) rather than writing once up front, mirroring how the real
        // `ingest_loop` reconciles: it re-checks and rewrites the desired
        // target on every reading until the device confirms it, rather
        // than writing exactly once. This matters here because opening a
        // fresh connection resets a genuine Arduino Uno via DTR, and the
        // bootloader silently discards serial bytes received during its
        // ~1-2s post-reset window before the sketch's `loop()` starts — a
        // single write issued immediately after open can be lost entirely.
        // Retrying on each iteration guarantees at least one write lands
        // once the sketch is actually listening.
        source
            .write_target(target)
            .await
            .expect("write_target should succeed");

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

/// The Arduino firmware prints a JSON line every ~10s. After reading one
/// line (consuming one print cycle), the NEXT line should arrive within
/// ~10s — proving the serial port stayed open and was not unnecessarily
/// closed and reopened between reads. An 11s timeout comfortably fits the
/// ~10s print interval while catching any unexpected reconnect delay (which
/// would add ~2s of setup on top of the print cycle).
///
/// This validates the "keep stream alive on transient errors" behaviour:
/// after the fix, read/write errors no longer drop the stream (only EOF
/// does), so a second read on the same source does not trigger a
/// DTR-reset-prone reopen.
const CONSECUTIVE_READ_TIMEOUT: Duration = Duration::from_secs(11);

#[tokio::test]
#[ignore]
async fn consecutive_reads_within_print_interval() {
    let _guard = HARDWARE_LOCK.lock().await;
    let mut source = ArduinoSerialSource::new(test_port(), test_baud());

    // Read one line first to establish the connection (the first open
    // always resets the Arduino — unavoidable kernel-level DTR assertion).
    let Ok(Ok(first)) = tokio::time::timeout(READ_TIMEOUT, source.read_line()).await else {
        panic!("expected first read to succeed within {READ_TIMEOUT:?}");
    };
    assert!(!first.is_empty(), "expected a non-empty first line");

    // Now read a second line. If the stream was kept alive (no unnecessary
    // reopen between reads), this arrives within the normal ~10s print
    // interval. If the stream was dropped somewhere, a reopen would
    // trigger a DTR reset and the line would take ~12-13s, missing our
    // tight timeout.
    let Ok(Ok(second)) = tokio::time::timeout(CONSECUTIVE_READ_TIMEOUT, source.read_line()).await
    else {
        panic!(
            "second read took longer than {CONSECUTIVE_READ_TIMEOUT:?} — \
             the stream was likely closed between reads, causing a DTR-reset \
             on reopen.  The fix keeps the stream alive on transient errors \
             so this should not happen."
        );
    };
    assert!(!second.is_empty(), "expected a non-empty second line");
}
