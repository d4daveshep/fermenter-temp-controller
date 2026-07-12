## Context

Slices 1-5 implemented the full vertical stack (ingest, persistence,
dashboard, target control, brew sessions) driving `AppState` from a
`MockSerialSource` that plays back scripted lines and records writes in
memory (`docs/rewrite-plan.md` §2, §12.3). The `SerialSource` trait and the
wire contract it must honor (115200 baud, newline-delimited JSON in,
`<value>`-framed float out, reconnect/backoff) were fixed in slice-1
(`openspec/specs/device-connection/spec.md`) specifically so a later slice
could swap the implementation without touching any dependent code
(`ingest`, `state`, `web`). This slice is that swap.

The physical test rig for this slice is a spare Arduino running the current
`TempController` firmware (see `arduino/TempController/TempController.ino`),
connected via USB serial directly to the dev machine — no sensors attached.
The firmware boots and emits well-formed JSON on schedule regardless of
sensor presence: `sensors.getTempCByIndex(n)` returns the DallasTemperature
library's `DEVICE_DISCONNECTED_C` sentinel (`-127.0`) when no OneWire device
answers, so `instant`/`average`/`min`/`max`/`ambient` will read `-127`-ish
but the JSON shape, `reason-code`, and `json-size` fields are unaffected.
That's sufficient for this slice, whose tests only assert on serial framing
(a line was read, a target write round-tripped) — not on temperature
plausibility.

## Goals / Non-Goals

**Goals:**
- Implement `ArduinoSerialSource` (in `src/serial/arduino.rs`) using
  `tokio-serial`, satisfying the existing `SerialSource` trait exactly as
  the mock does today.
- Wire `MOCK_SERIAL` in `main.rs` so `false` (the documented default,
  `docs/rewrite-plan.md` §9) constructs `ArduinoSerialSource` against
  `SERIAL_PORT`/`SERIAL_BAUD` instead of panicking or falling through to the
  mock.
- Implement the reconnect/backoff loop (open failure, mid-read I/O error, or
  unplug) against a real port, fulfilling the `device-connection` "Serial
  connection resilience" requirement for the first time under real I/O
  conditions.
- Add `#[ignore]`'d hardware tests (`tests/serial_hardware.rs`) that open the
  real port, read one real line, and round-trip a target write, runnable via
  `cargo test -- --ignored` against the spare Arduino on the dev machine.

**Non-Goals:**
- No change to the `device-connection` contract itself — it was already
  fully specified in slice-1; this slice only proves an alternate
  implementation against it. No spec deltas are expected
  (`docs/openspec-rewrite-management.md` §2/§3).
- No Arduino firmware changes.
- No Raspberry Pi deployment or `docker compose` device passthrough
  verification — that's slice-7 (milestones 7-8).
- No assertion on temperature *values* being sensible; sensors are out of
  scope for this slice's tests.
- No CI execution of the hardware tests — they stay `#[ignore]`'d
  (`docs/rewrite-plan.md` §12.6).

## Decisions

- **`tokio-serial` for the real transport** — already the dependency named
  in `docs/rewrite-plan.md` §14 and the natural async-native counterpart to
  `mpsc`/`watch`-based `SerialTask`; avoids introducing a second async
  runtime bridge (e.g. blocking `serialport` crate + `spawn_blocking`).
- **Reuse the existing `SerialSource` trait unchanged** — `read_line` /
  `write_target` are already exercised end-to-end by the mock through
  ingest/state/web; `ArduinoSerialSource` is a drop-in implementation, so no
  call sites change. This is what keeps the spec delta empty.
- **Reconnect/backoff lives inside `ArduinoSerialSource` (or a thin wrapper
  around it), not in the caller** — the `SerialTask` loop (`docs/rewrite-plan.md`
  §5) just calls `read_line`/`write_target` and treats a returned error as
  "try again after backoff"; the exponential backoff-with-cap policy is
  implemented once, near the port-open logic, so it's exercised identically
  whether the failure is "port never opened" or "port dropped mid-stream."
- **Hardware tests are `#[ignore]`'d integration tests, not unit tests** —
  per the layered test strategy (`docs/rewrite-plan.md` §12.1/§12.4), keeping
  them out of `src/` and out of the default `cargo test` run means CI (which
  has no attached Arduino) is unaffected, while `cargo test -- --ignored` on
  a machine with the device attached exercises the real contract.
- **No sensor dependency for this slice's verification** — the hardware
  tests assert on framing (a JSON line arrived, a `<target>` write produced
  the expected reported target on a subsequent line), which the firmware
  satisfies with or without DallasTemperature sensors physically present.
  Confirmed against `TempController.ino`: `sensors.begin()` /
  `requestTemperatures()` / `getTempCByIndex()` tolerate zero discovered
  OneWire devices and return `-127.0` rather than blocking or crashing.

## Risks / Trade-offs

- **[Risk] Serial port path is host-specific** (e.g. `/dev/ttyACM0` vs.
  `/dev/ttyUSB0`, or a different path entirely on the dev machine vs. the
  eventual Pi) → **Mitigation:** `SERIAL_PORT` is already an env var
  (`docs/rewrite-plan.md` §9); the hardware test reads it from the
  environment (with a sensible default) rather than hardcoding a path, and
  the test is skipped/fails fast with a clear message if the device isn't
  present.
- **[Risk] Without sensors, firmware may report `-127.0` continuously and
  the ambient/fermenter action-decision logic could look pathological if
  someone eyeballs the dashboard during manual testing** → **Mitigation:**
  explicitly out of scope for this slice (see Non-Goals); documented so
  nobody mistakes `-127.0` readings for a bug during slice-6 verification.
- **[Risk] Real serial I/O timing differs from the mock (e.g. partial lines,
  OS buffering, USB re-enumeration on unplug) in ways the mock never
  exercised** → **Mitigation:** this is precisely why slice-6 exists as a
  dedicated slice rather than folding `arduino.rs` into slice-1; the
  `#[ignore]`'d tests are the first real-world check before Pi deployment.
- **[Risk] Developer's dev machine may not expose the Arduino at a
  predictable device path across reconnects (common on Linux with multiple
  USB-serial adapters)** → **Mitigation:** document using `/dev/serial/by-id/...`
  or udev symlink in the test's setup notes rather than assuming
  `/dev/ttyACM0`.

## Migration Plan

Not applicable — this slice adds a new implementation behind an existing
trait and a config flag that already defaults to using it
(`MOCK_SERIAL=false`); there's no data migration, and rollback is simply
setting `MOCK_SERIAL=true` to fall back to the mock.

## Open Questions

- Exact `tokio-serial` reconnect strategy (poll-and-retry vs. OS-level
  hotplug notification) — default to poll-and-retry with the backoff policy
  already specified for `device-connection`; revisit only if it proves
  insufficient in practice.
- Whether the hardware test asserts on the JSON *shape* only, or also
  round-trips a `<target>` write and waits for the next reading to report it
  — current intent (per `docs/rewrite-plan.md` §12.4) is both; confirmed in
  tasks.md.
</content>
