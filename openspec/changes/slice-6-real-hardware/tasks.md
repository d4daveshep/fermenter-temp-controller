## 1. Dependencies & scaffolding

- [ ] 1.1 Add `tokio-serial` to `Cargo.toml`.
- [ ] 1.2 Create `src/serial/arduino.rs` module, `mod arduino;` in `src/serial/mod.rs`.

## 2. Hardware tests FIRST (RED)

- [ ] 2.1 Write `tests/serial_hardware.rs`, all `#[ignore]`, reading
      `SERIAL_PORT`/`SERIAL_BAUD` from the environment (default
      `/dev/ttyACM0` @ `115200` if unset):
  - [ ] 2.1.1 `#[ignore] fn opens_real_port()` — open the configured port,
        assert success.
  - [ ] 2.1.2 `#[ignore] fn reads_a_real_line()` — open the port, call
        `read_line()`, assert the returned string is non-empty, ends where
        expected (no embedded newline), and parses as JSON (don't assert on
        field values — no sensors attached, so `instant`/`average`/`min`/
        `max`/`ambient` may read `-127`-ish per `DEVICE_DISCONNECTED_C`).
  - [ ] 2.1.3 `#[ignore] fn write_target_roundtrips()` — call
        `write_target(x)`, then poll `read_line()` a few times until a
        parsed reading reports `target == x`, with a timeout so the test
        fails cleanly instead of hanging if the firmware doesn't ack.
  - [ ] 2.1.4 Confirm all three fail to compile/run against a stub (RED)
        before `arduino.rs` exists.

## 3. `ArduinoSerialSource` implementation (GREEN)

- [ ] 3.1 Implement `SerialSource` for `ArduinoSerialSource` using
      `tokio-serial`: open the port at construction (or lazily on first
      use), `read_line` reads until `\n` and returns the line as UTF-8
      (matching the mock's contract), `write_target` writes the value
      framed as `<value>` per the fixed firmware contract
      (`openspec/specs/device-connection/spec.md`).
- [ ] 3.2 Run `cargo test -- --ignored` against the spare Arduino (USB to
      dev machine, no sensors required) and get 2.1.1-2.1.3 to GREEN.

## 4. Reconnect / backoff

- [ ] 4.1 Implement exponential backoff (capped) around port-open failures
      and mid-read I/O errors, per the `device-connection` "Serial
      connection resilience" requirement.
- [ ] 4.2 Add an `#[ignore]`'d test that unplugs (or the human tester
      manually disconnects) the Arduino mid-test and confirms the log shows
      increasing backoff delays and eventual reconnect once replugged.
      (Manual step noted in test doc-comment since USB unplug can't be
      automated without extra hardware.)

## 5. Wire `MOCK_SERIAL`

- [ ] 5.1 In `main.rs`, branch on `MOCK_SERIAL`: `true` → existing
      `MockSerialSource`, `false` → new `ArduinoSerialSource` built from
      `SERIAL_PORT`/`SERIAL_BAUD`.
- [ ] 5.2 Confirm `cargo test` (default, no `--ignored`) still passes
      unchanged — hardware tests stay excluded from the default run.

## 6. Manual end-to-end verification (dev machine)

- [ ] 6.1 Plug the spare Arduino (latest firmware, no sensors) into the dev
      machine via USB; note the assigned device path
      (`/dev/serial/by-id/...` preferred over `/dev/ttyACM0` if multiple
      USB-serial devices are present).
- [ ] 6.2 Run `cargo test -- --ignored` and confirm all `serial_hardware.rs`
      tests pass.
- [ ] 6.3 Run the binary with `MOCK_SERIAL=false` pointed at the Arduino;
      confirm readings ingest (values will show `~-127` for temperature
      fields — expected without sensors) and a target write via the
      dashboard round-trips into a subsequent reading's `target` field.
- [ ] 6.4 Unplug the Arduino while the binary is running; confirm
      reconnect/backoff logs appear and ingestion resumes cleanly on
      replug.

## 7. Spec & docs

- [ ] 7.1 `openspec validate slice-6-real-hardware` passes.
- [ ] 7.2 Archive the change once 2-6 are complete and verified, folding the
      `device-connection` scenario additions into
      `openspec/specs/device-connection/spec.md`.
</content>
