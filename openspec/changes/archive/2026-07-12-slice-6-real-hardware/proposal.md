## Why

Slices 1-5 built and proved the entire controller (ingest, persistence,
dashboard, target control, brew sessions) against a `MockSerialSource`. The
`device-connection` capability's contract (115200 baud, newline-delimited
JSON in, `<value>`-framed float out, reconnect/backoff) was fixed in slice-1
but has never been exercised against a real Arduino. Milestone 6 of
`docs/rewrite-plan.md` calls for swapping the mock for a real `tokio-serial`
implementation; this slice is what proves that swap is safe before the Pi
deployment work in slice-7.

## What Changes

- Add `serial/arduino.rs`: a `SerialSource` implementation backed by
  `tokio-serial`, opening the configured `SERIAL_PORT` at `SERIAL_BAUD` and
  implementing the same `read_line` / `write_target` contract as the mock.
- Wire the existing `MOCK_SERIAL` config flag so `main.rs` selects
  `ArduinoSerialSource` when `MOCK_SERIAL=false` (already the documented
  default) instead of failing/no-oping.
- Implement the reconnect/backoff loop against the real port (open failure,
  mid-read I/O error, or device unplug all trigger the same exponential
  backoff already specified for `device-connection`).
- Add `tests/serial_hardware.rs`: `#[ignore]`'d hardware tests that open a
  real serial port, read one real line, and round-trip a target write. These
  never run in CI; they're invoked manually with `cargo test -- --ignored`
  against a physically attached Arduino.
- No changes to the Arduino firmware. No changes to any other capability's
  behavior — this slice only proves the existing `device-connection` contract
  against real hardware.

## Capabilities

### New Capabilities

(none)

- `device-connection`: no existing requirement text changes — this slice
  fulfills the contract already specified in slice-1 by swapping its
  implementation from mock to real `tokio-serial`. Per
  `docs/openspec-rewrite-management.md` §2/§3, slice-6 is expected to carry
  effectively zero spec deltas. The only delta here adds one new scenario to
  each of "Serial transport abstraction" and "Serial connection resilience"
  documenting that the existing behavior is now verified against real
  hardware — no existing scenario's wording changes.

## Impact

- **Code:** new `src/serial/arduino.rs`; `main.rs` gains the `MOCK_SERIAL`
  branch that constructs it; new `tests/serial_hardware.rs`.
- **Dependencies:** `tokio-serial` (already listed as a target dependency in
  `docs/rewrite-plan.md` §14; not yet added to `Cargo.toml`).
- **Hardware:** requires one Arduino running the current `TempController`
  firmware, connected via USB to whichever machine runs
  `cargo test -- --ignored` (dev machine is sufficient for this slice;
  Raspberry Pi deployment is out of scope, covered by slice-7).
  Temperature sensors are NOT required for this slice — the hardware tests
  only exercise serial framing (line read, `<target>` write roundtrip), not
  reading accuracy. Without sensors attached, the firmware still emits
  well-formed JSON lines using the DallasTemperature library's
  `DEVICE_DISCONNECTED_C` sentinel (`-127.0`) for `instant`/`average`/`min`/
  `max`/`ambient`.
- **CI:** no effect — hardware tests are `#[ignore]`'d and excluded from the
  default `cargo test` run per `docs/rewrite-plan.md` §12.6.
</content>
