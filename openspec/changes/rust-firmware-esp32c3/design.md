## Context

The fermenter controller board is an Arduino Uno running C++ firmware
(`arduino/TempController/`). The Uno cannot run Rust. The Rust Axum host
(`fermenter/`) treats the board as a black box over USB serial: it reads
newline-delimited JSON telemetry and writes `<float>`-framed target temperatures
back. The host's `SerialSource` trait and all dependent code remain unchanged
by this rewrite — only the concrete device on the other end of the cable
changes.

The replacement board is an **Espressif ESP32-C3 SuperMini Dev Board** with native
USB-Serial-JTAG. The ESP32-C3's USB-Serial-JTAG peripheral connects directly to
the USB-C connector through the chip's internal USB PHY — it uses no external
CH340/CP2102 bridge and occupies no numbered GPIO pins. The physical header pins
GPIO20 (RX) and GPIO21 (TX) are the separate UART0 port, available for other use
if needed. Target Rust toolchain: `riscv32imc-unknown-none-elf` via `espup`,
flashed with `espflash`. The board runs Embassy's cooperative async executor,
which is the established pattern for Rust on embedded systems where `tokio` is
unavailable.

The hardware scope is narrower than the original: the LCD and its 6 GPIO pins
are gone. Three signals remain — OneWire data bus (one shared data line for both
DS18B20 sensors), and two relay control outputs (heat, cool).

## Goals / Non-Goals

**Goals:**

- Port all controller logic (RC1–RC10 decision rules, EMA, relay model) to Rust,
  fully unit-testable on the host without embedded toolchain or hardware.
- Implement the identical serial protocol (`fermenter/` requires zero code changes).
- Fix the two known Arduino firmware quirks: zero-target suppression and
  positional sensor identity.
- Provide a clear flash-and-run path for the test board and production board.
- All logic paths covered by tests that run under plain `cargo test` before
  any hardware is needed.

**Non-Goals:**

- WiFi/network telemetry transport (deferred; a future change will add this when needed).
- Multi-task Embassy split (deferred; documented as a future refactor below).
- LCD support (removed from hardware build).
- Any changes to `fermenter/` host code.

## Decisions

### Decision 1 — Two-crate Cargo workspace at `firmware/`

**Choice:** `firmware/` is a standalone Cargo workspace with two crates:
`firmware/logic/` (pure library) and `firmware/esp32c3/` (no_std binary).

**Rationale:** Keeping logic separate from the hardware crate means:

- `logic/` compiles and tests under standard `cargo test` on any host, with no
  embedded toolchain, no `probe-rs`, no hardware. The Arduino C++ test suite
  (AUnit, ~40 cases across `Test_ControllerActionRules.cpp`,
  `Test_Decision.cpp`, `Test_TemperatureReadings.cpp`) is ported here as
  native `#[test]`/`rstest` cases — regression parity from day one.
- `esp32c3/` can `dev-depend` on `logic/` without polluting the host
  `fermenter/` workspace (different target triple, different toolchain channel).

`firmware/` is intentionally **not** a member of `fermenter/`'s Cargo workspace.
The two workspaces share the repo but build independently.

**Alternative considered:** Single flat crate with `cfg(target_arch = "riscv32")`.
Rejected — makes local test runs depend on the embedded target being installed,
friction for CI and contributors.

### Decision 2 — Single-task Embassy loop for v1

**Choice:** One `#[embassy_executor::task]` replicating the Arduino `loop()`
cadence: tick (1s via `embassy_time::Timer`), poll for `<target>` bytes on USB
serial, read both DS18B20s, `make_action_decision`, set relay GPIOs, emit JSON
telemetry every ~10s.

**Rationale:** The Arduino firmware's single-loop design has been proven reliable
in production. Matching it exactly for v1 minimises behavioral divergence and
makes fault isolation straightforward during hardware bring-up on a brand-new
board. Embassy's cooperative executor gives deterministic task scheduling without
the complexity of cross-task `Signal`/`Mutex` for a first version.

**Future refactor (documented, not built):** Split into:

- `sensor_task` — owns the OneWire bus, pushes readings via `Signal`
- `serial_rx_task` — owns USB serial RX, parses `<target>` frames, sends via
  `Signal`
- `serial_tx_task` — owns USB serial TX, receives JSON telemetry via `Signal`

This separation decouples the ~750ms DS18B20 conversion wait from serial
responsiveness. Worth doing once the v1 loop is validated on hardware.

### Decision 3 — ROM-address sensor identification

**Choice:** Each DS18B20 is identified by its unique 64-bit ROM address, not
by bus-scan index. The ROM address is discovered once (via a helper binary or
`espflash` log during bring-up) and stored as compile-time constants
`FERMENTER_SENSOR_ADDR` and `AMBIENT_SENSOR_ADDR`.

**Rationale:** `getTempCByIndex(0)`/`(1)` in the Arduino firmware assigns
"fermenter" and "ambient" roles based on the order the `DallasTemperature`
library happens to enumerate sensors on the bus. This is fragile: if a sensor
is replaced or the cable is reseated, the roles silently swap, which corrupts
the control loop (a heater is told to cool a fermenter that's actually reading
ambient, and vice versa). ROM addresses are unique and invariant across bus
rescans.

**Alternative:** Keep index-based for parity. Rejected — the fragility was
explicitly identified as a known quirk to fix in this rewrite.

### Decision 4 — Fix zero-target suppression

**Choice:** The `<target>` frame parser returns `Option<f64>` — `None` when no
valid `<...>` frame is present in the received bytes, `Some(0.0)` when the
frame contains the literal value `0.0`.

**Rationale:** The original C++ does `if (newTarget != 0.0) { set target; }`,
meaning a received `<0.0>` is silently ignored. The host `fermenter/` already
allows `TARGET_MIN = 0.0` as a valid operator-set temperature (see
`fermenter/src/temperature_control.rs`), so this Arduino quirk means lagering
at 0°C could never be achieved. The Rust parser removes this ambiguity cleanly
with `Option<f64>`.

### Decision 5 — JSON encoding via `serde-json-core`

**Choice:** Use `serde-json-core` (no_std, heapless, stack-allocated) for
serialising the outgoing JSON telemetry. Use `heapless::String` for the
formatted output.

**Rationale:** `serde_json` requires `alloc`. `serde-json-core` is the
established no_std alternative and matches the project's existing `serde` usage.

**Fields emitted** (minimal set that satisfies `fermenter/src/model.rs`):

```json
{
  "target": 19.5,
  "average": 18.2,
  "min": 18.0,
  "max": 18.4,
  "ambient": 20.1,
  "action": "Rest",
  "reason-code": "RC3.1"
}
```

Fields dropped vs. the Arduino: `instant`, `json-size`, and the
`rest`/`heat`/`cool` boolean flags — none are consumed by the host. `action` and
`reason-code` must match exactly (the host's `reason_code.rs` pattern-matches
on them).

### Decision 6 — Native USB-Serial-JTAG

**Choice:** Use the ESP32-C3's native USB-Serial-JTAG peripheral as the serial
transport to the Pi host. No CH340/CP2102 bridge chip.

**Rationale:** The ESP32-C3 SuperMini's USB-Serial-JTAG peripheral is connected
to the USB-C port via the chip's internal USB PHY — it has no associated GPIO
pin number (it is entirely internal to the SoC). It enumerates as a CDC-ACM
device (`/dev/ttyACM0` on Linux), indistinguishable to the host from the Arduino
Uno's FTDI bridge. Note: GPIO20 (RX) and GPIO21 (TX) on the header are a
separate UART0 peripheral; they play no role in the host-facing serial link.

On reset, the USB-Serial-JTAG peripheral re-enumerates cleanly — there is no
DTR pulse on physical GPIO pins — so the host's existing EOF-only reconnect
strategy (`fermenter/src/serial/arduino.rs`) handles reconnection without any
code changes.

### Decision 7 — `esp-println` for diagnostic output during bring-up

**Choice:** Use `esp-println` for `println!`-style diagnostic output during
development and hardware bring-up. Switch to silent operation (or a structured
log over USB) once stable.

**Rationale:** Lightweight, works with USB-Serial-JTAG out of the box, no RTT
probe required.

## Risks / Trade-offs

**[ROM address discovery is a manual step]** → The `FERMENTER_SENSOR_ADDR` and
`AMBIENT_SENSOR_ADDR` constants must be determined by running a discovery helper
on the actual hardware before the production binary can be compiled. Mitigation:
document the discovery procedure in `firmware/README.md`; provide a
`sensor-discovery` binary or example in `firmware/esp32c3/` that prints ROM
addresses and exits.

**[OneWire timing on ESP32-C3]** → The `one-wire-bus` crate relies on
`embedded-hal` blocking delays; timing correctness on the ESP32-C3 depends on
`esp-hal`'s delay implementation matching the DS18B20's protocol requirements.
Mitigation: covered by hardware integration test (board + real sensors);
flag as the first thing to verify during bring-up.

**[Async serial I/O in Embassy]** → Embassy on ESP32-C3 has async USB-Serial-JTAG
support, but the exact API surface (whether `UsbSerialJtag` implements
`embedded-io-async` traits or requires a wrapper) needs confirming against the
`esp-hal` version in use. Mitigation: lock `esp-hal` version in `Cargo.toml`;
verify with a "hello world" serial echo before implementing the protocol.

**[No heap allocator]** → All buffers (JSON output, serial receive ring) must be
statically or stack-allocated. `serde-json-core` and `heapless` handle this, but
JSON output size must be bounded. Worst-case JSON measurement is ~130 bytes
(Arduino emitted `json-size` as a self-check); a `heapless::String<256>` is
sufficient headroom. Mitigation: assert at compile time or test that the maximum
expected JSON fits within the chosen buffer size.

**[Cross-compilation toolchain setup]** → Requires `espup`, the RISC-V GCC
toolchain, and `espflash`. Not part of the standard Rust toolchain. Mitigation:
document setup steps in `firmware/README.md`; CI build job verifies compilation
compiles successfully (flash step is hardware-only).

## Migration Plan

1. Flash the **test board** with the new firmware; validate sensor reads, relay
   operation, and serial JSON output against the existing `fermenter/` host
   (mock-serial off, real board plugged in).
2. Verify no changes needed in `fermenter/.env` (`SERIAL_PORT` may need
   updating if the ESP32-C3 enumerates at a different path — confirm on the Pi).
3. Flash the **production board** and swap it in for the Arduino Uno.
4. `arduino/TempController/` stays in the repo; no deletion needed.

**Rollback:** Re-flash the Arduino Uno with the original C++ firmware using
`upload_arduino.sh`. No host-side changes to undo.

## Open Questions

- Which GPIO pins should be used for the relay outputs and OneWire data line?
  The board exposes GPIO0–10, GPIO20, and GPIO21 on its headers (13 pins total).
  Strapping pins GPIO2, GPIO8, and GPIO9 must be avoided for relay and sensor
  outputs. USB-Serial-JTAG uses no GPIO numbers (internal USB PHY). GPIO20/21
  (UART0) are available for general use but should be reserved in case UART0 is
  needed later. That leaves GPIO0, GPIO1, GPIO3, GPIO4–7, and GPIO10 as the
  candidates — the safest first picks with no system duties are GPIO0, GPIO1,
  GPIO3, and GPIO10. The exact assignments will be finalized as named compile-time
  constants before first flash.
  _(To be resolved in task 12.1 of tasks.md — GPIO pin assignment and relay control.)_
