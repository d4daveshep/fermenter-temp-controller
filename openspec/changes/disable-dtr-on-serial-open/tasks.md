## 1. Hardware regression test (RED)

- [ ] 1.1 Add a new `#[ignore]`'d test `reconnect_does_not_reset_arduino` to
      `fermenter/tests/serial_hardware.rs`:
  - [ ] 1.1.1 Open a first `ArduinoSerialSource` against the configured
        port, call `read_line()` once (reusing the existing
        `HARDWARE_LOCK` guard pattern so it doesn't race the other
        hardware tests for the exclusive port lock).
  - [ ] 1.1.2 Drop the first source (closing its stream), then construct a
        *second* `ArduinoSerialSource` against the same port and call
        `read_line()` again.
  - [ ] 1.1.3 Assert the second `read_line()` returns within a timeout
        tight enough that a DTR-triggered reset would miss it (e.g. 15s —
        shorter than the ~2s setup delay stacked onto the firmware's ~10s
        print cycle that a reset causes, per `design.md` decision 3), while
        still comfortably above the no-reset case (~10s, the plain print
        cycle).
  - [ ] 1.1.4 Document in a doc comment that this test requires a real,
        physically attached Arduino (no sensors needed — only timing is
        asserted, matching `opens_real_port`/`reads_a_real_line`'s existing
        convention) and is expected to FAIL against today's code because
        `tokio_serial::new()` asserts DTR by default.
- [ ] 1.2 Run `cargo test --test serial_hardware -- --ignored
      --test-threads=1 reconnect_does_not_reset_arduino` against the
      spare/test Arduino and confirm it FAILS (RED) — the second connection
      should take noticeably longer than 15s due to the DTR-triggered
      reset.

## 2. Implement DTR/RTS disable (GREEN)

- [ ] 2.1 In `ArduinoSerialSource::ensure_connected`
      (`fermenter/src/serial/arduino.rs`), add `.set_dtr(false)` and
      `.set_rts(false)` to the `tokio_serial::new(&self.port_path,
      self.baud_rate)` builder chain, before `.open_native_async()`.
- [ ] 2.2 Re-run `cargo test --test serial_hardware -- --ignored
      --test-threads=1 reconnect_does_not_reset_arduino` against the same
      Arduino and confirm it now PASSES (GREEN) — the second connection's
      first line arrives within the tight timeout.

## 3. Confirm no regressions

- [ ] 3.1 Run `cargo test --test serial_hardware -- --ignored
      --test-threads=1` (full file) and confirm `opens_real_port`,
      `reads_a_real_line`, and `write_target_roundtrips` still pass — their
      generous existing timeouts (20s read, 60s round-trip) remain valid
      whether or not a reset occurs, so they should be unaffected either
      way.
- [ ] 3.2 Run `cargo test` (default, no `--ignored`) from `fermenter/` and
      confirm the full non-hardware suite still passes unchanged,
      including `arduino::tests::backoff_doubles_on_repeated_open_failure_then_caps`
      (asserts backoff timing against a nonexistent port; unaffected by
      DTR/RTS settings since that path never successfully opens a port).

## 4. Lint, types, full suite

- [ ] 4.1 Run `cargo fmt --check` from `fermenter/` — clean
- [ ] 4.2 Run `cargo clippy --all-targets -- -D warnings` from `fermenter/`
      — clean
- [ ] 4.3 Run `cargo test` from `fermenter/` — full default suite GREEN

## 5. OpenSpec validation

- [ ] 5.1 Run `openspec validate disable-dtr-on-serial-open --strict` —
      passes (confirms the ADDED delta against `device-connection` is
      well-formed, requirement/scenario headers correct)
- [ ] 5.2 Run `openspec status --change "disable-dtr-on-serial-open"` —
      `isComplete: true`

## 6. Manual verification on the real Pi/Arduino deployment

- [ ] 6.1 Deploy the change to the Raspberry Pi (or run it locally pointed
      at the real serial device) and confirm the Arduino does not reset
      (no LCD flicker/reinit, no ~12s startup delay in logs) when the app
      starts or reconnects after a transient serial error.
- [ ] 6.2 Over a period spanning at least one natural temperature swing,
      confirm the dashboard's `min`/`max` stay consistent with the range
      visible on the chart for the same window (the original symptom this
      change fixes).
</content>
