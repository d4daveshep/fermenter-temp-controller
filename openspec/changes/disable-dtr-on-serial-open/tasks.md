## 1. Hardware regression test

- [x] 1.1 Add a new `#[ignore]`'d test `consecutive_reads_on_same_source`
      to `fermenter/tests/serial_hardware.rs`:
  - [x] 1.1.1 Open a single `ArduinoSerialSource`, read one line with the
        existing `READ_TIMEOUT` (20s).
  - [x] 1.1.2 Read a SECOND line from the same source with `READ_TIMEOUT`.
        If the stream stays alive (no unnecessary reopen), the second read
        completes normally. If the stream was dropped between reads, a
        reopen would reset the Arduino and the test would time out.
- [x] 1.2 Run `cargo test --test serial_hardware -- --ignored
      --test-threads=1 consecutive_reads_on_same_source` against the
      spare/test Arduino — PASSES

## 2. Keep stream alive on transient errors

- [x] 2.1 In `ArduinoSerialSource::read_line` (`fermenter/src/serial/arduino.rs`),
      remove the `self.stream = None` / backoff / sleep block from the read
      `Err(e)` branch. Log the error at `warn!` and return it to the caller
      without closing the stream. Only the `Ok(0)` (EOF) branch drops the
      stream and runs backoff.
- [x] 2.2 In `ArduinoSerialSource::write_target`, remove `self.stream = None`
      from both the write error and flush error branches. Log and return the
      error while keeping the stream alive.
- [x] 2.3 Add `.dtr_on_open(false)` to the `tokio_serial::new(...)` builder
      chain before `.open_native_async()` in `ensure_connected`.
- [x] 2.4 Run `cargo build` — compiles cleanly

## 3. Confirm no regressions

- [x] 3.1 Run `cargo test` (default, no `--ignored`) from `fermenter/` and
      confirm the full non-hardware suite passes unchanged, including:
  - `serial::arduino::tests::backoff_doubles_on_repeated_open_failure_then_caps`
    (backoff timing against nonexistent port — unaffected)
  - All mock-serial and ingest tests (use `MockSerialSource`, not the real
    transport)
- [x] 3.2 Run `cargo test --test serial_hardware -- --ignored
      --test-threads=1` (full file) — `opens_real_port`,
      `reads_a_real_line`, `write_target_roundtrips`, and
      `consecutive_reads_on_same_source` all PASS

## 4. Lint, types, full suite

- [x] 4.1 Run `cargo fmt --check` from `fermenter/` — clean
- [x] 4.2 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/`
      — clean
- [x] 4.3 Run `cargo test` from `fermenter/` — full default suite GREEN

## 5. OpenSpec validation

- [x] 5.1 Run `openspec validate disable-dtr-on-serial-open --strict` —
      passes (confirms the ADDED delta against `device-connection` is
      well-formed, requirement/scenario headers correct)
- [x] 5.2 Run `openspec status --change "disable-dtr-on-serial-open"` —
      `isComplete: true`

## 6. Verification on the real Pi/Arduino deployment

- [x] 6.1 Deploy to the Pi, restart the Docker container — Arduino resets on
      every restart. **Confirmed expected**: the kernel asserts DTR during
      `open()` (USB CDC ACM protocol), so process restarts always reset the
      Arduino. This cannot be prevented in software. The fix only protects
      against resets during *active use* (transient errors no longer cause
      reopens).
- [x] 6.2 **Cancelled** — the original symptom (dashboard `min` vs chart
      discrepancy) was caused by transient-error-triggered reconnects during
      the same session, not by the startup reset. The fix addresses this,
      but the user declined to pursue a hardware fix for the startup reset,
      so prolonged observation is moot without it.
