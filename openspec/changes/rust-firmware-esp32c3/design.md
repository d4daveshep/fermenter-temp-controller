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

**Addendum (task 12):** `firmware/esp32c3` itself is *both* a library
(`src/lib.rs`, exposing `relays`/`sensors`/etc.) and a binary (`src/main.rs`).
Cargo auto-detects this without any extra `Cargo.toml` section, and `main.rs`
accesses the shared modules as `use esp32c3::{relays, sensors};` (the
package's own name, no different from any other dependency). This lets
`examples/*.rs` (each a separate binary target — `discover_sensors.rs`,
`relay_test.rs`, and more to come in later tasks) reuse the same
`RelayController`/sensor code as the real firmware instead of duplicating it
per example.

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

**Runtime crate and entry point:** The executor itself is provided by
**`esp-rtos`** (with its `embassy` feature enabled), not by the older
`esp-hal-embassy` crate — `esp-hal-embassy` was merged into `esp-rtos` as of
`esp-rtos` v0.1.0 and has had no releases since 0.9.1 (2025-10-14), while
`esp-rtos` (0.3.0 as of this writing) is the actively maintained successor and
pairs with the `esp-hal ~1.1` pin from Decision 6. `main.rs` is written as:

```rust
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(config);
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);
    spawner.spawn(controller_task(...)).unwrap();
}
```

`#[esp_rtos::main]` only wraps the function body in a thread-mode executor —
it does **not** start the runtime for you. `esp_rtos::start(timer,
software_interrupt)` must be called manually, first thing inside `main`,
before spawning any task. This claims a hardware timer group (`TIMG0`) and the
`FROM_CPU0` software interrupt as runtime resources, in addition to the GPIO
pins discussed in Open Questions below. (`#[esp_hal::main]` is the same
underlying macro re-exported from `esp-hal` instead of `esp-rtos`; either
attribute path works, so long as `esp-rtos` is a real Cargo dependency.)

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

**Fields deliberately omitted vs. the original Arduino firmware:**
- `instant` — the Arduino emitted a `instant` field containing the raw
  (non-averaged) current temperature reading. The host's `fermenter/src/model.rs`
  `Reading` struct has no `instant` field and never reads it; it was a firmware
  debug convenience. Omitting it keeps the telemetry payload smaller and removes
  dead data. A developer porting the Arduino code should not add it back.
- `json-size` — a self-diagnostic field the Arduino used to verify JSON buffer
  sizing; accepted but ignored by the host (`#[serde(default)]`). Irrelevant
  in the Rust firmware where buffer sizing is compile-time checked.
- `rest`/`heat`/`cool` boolean flags — redundant with the `action` string field;
  not used by any host code or template.

`action` and `reason-code` must match exactly (the host's `reason_code.rs`
pattern-matches on the RC codes; the dashboard template renders `action`
directly).

### Decision 6 — Serial transport: USB-Serial-JTAG via `esp-hal unstable` feature

**Choice:** Use the ESP32-C3's native USB-Serial-JTAG peripheral as the serial
transport to the Pi host, accessed via `esp-hal`'s `usb_serial_jtag` module.
This requires enabling the `unstable` *cargo feature* of `esp-hal`.

**Important distinction — stable Rust compiler vs. `unstable` cargo feature:**
These are two entirely separate things and must not be confused:

- **Rust compiler channel**: `esp-hal` v1.x fully supports RISC-V targets
  (including ESP32-C3) on **stable Rust**. No nightly compiler is needed.
  Espressif's own CI uses `toolchain: stable` for RISC-V builds.
- **`esp-hal`'s `unstable` cargo feature**: This is Espressif's internal API
  stability designation — a cargo feature flag that gates modules whose *API
  surface* the `esp-hal` team has not yet committed to stabilize across semver
  versions. It has nothing to do with the Rust compiler channel.

`usb_serial_jtag` is gated behind `esp-hal`'s `unstable` feature in v1.0 and
v1.1. This means the API may change in a future `esp-hal` minor release (though
not in a patch release). Given we will pin the `esp-hal` version in `Cargo.toml`
and update deliberately, this is an acceptable tradeoff.

**Alternative considered — UART0 on GPIO20/21 with an external adapter:**
The `uart` driver in `esp-hal` is stable (not gated by the `unstable` feature).
Using UART0 on header pins GPIO20 (RX) and GPIO21 (TX) would avoid the
`unstable` feature entirely, but requires an external USB-UART adapter (e.g.
CP2102 dongle) connected to those pins and plugged into the Pi — the USB-C port
would be used only for flashing, not for runtime telemetry. This is a meaningful
hardware cost and complicates the physical setup for no functional gain.

**Rationale for choosing USB-Serial-JTAG:**
- Single cable for both flashing and runtime telemetry (USB-C to Pi); cleaner
  physical setup.
- Enumerates as CDC-ACM (`/dev/ttyACM0` on Linux), indistinguishable to the
  host from the Arduino Uno's connection; no host-side changes needed.
- The USB-Serial-JTAG peripheral is connected to the USB-C port via the chip's
  internal USB PHY (no associated GPIO pin number). GPIO20/21 (UART0) remain
  available for other purposes.
- On reset, the peripheral re-enumerates cleanly (no DTR pulse on GPIO pins),
  matching the EOF-only reconnect strategy already in
  `fermenter/src/serial/arduino.rs`.
- Pinning `esp-hal = "~1.1"` in `Cargo.toml` bounds any `unstable` API churn
  to minor releases, which are deliberate upgrade decisions.

**Rollback path:** If the `usb_serial_jtag` API changes disruptively in a future
`esp-hal` release, switching to UART0 + external adapter is a straightforward
migration: change one peripheral in `main.rs`, update the Pi's `.env`
`SERIAL_PORT`, and add a CP2102 dongle to the hardware.

**Confirmed by implementation (task 10.1):** in the pinned `esp-hal 1.1.1`,
the module is the flat `esp_hal::usb_serial_jtag::{UsbSerialJtag,
UsbSerialJtagRx, UsbSerialJtagTx}` (not nested under a `usb::` module, which
appears in later/unreleased `esp-hal` versions). `UsbSerialJtag::new(...)
.into_async().split()` gives `(rx, tx)` implementing `embedded-io-async`'s
`Read`/`Write` traits directly — confirmed working end-to-end with a
single-task echo loop, compiled against the real dependency versions.

**Confirmed by hardware verification (task 10.2):** flashed to the actual
test board and exercised over its real `/dev/ttyACM0`; three round-tripped
messages (plain text, an RC-code-shaped string, and a `<19.5>`-framed value)
all echoed back byte-for-byte. One real behavior worth recording: immediately
after a reset, the ESP-IDF second-stage bootloader itself prints its own
boot-log text over the same USB-Serial-JTAG wire, *before* our application
even starts — this is chip/bootloader behavior, not something our own code
(or `debug-log`, Decision 7) controls, and it happens on every reset. It
needs no fix: `fermenter/src/ingest.rs`'s existing malformed-line handling
(`malformed_lines_are_skipped_loop_continues`) already logs and skips any
line that doesn't deserialize as a `Reading`, so this boot chatter is
tolerated exactly like any other non-JSON noise on the wire.

### Decision 7 — `esp-println` for diagnostic output during bring-up

**Choice:** Use `esp-println` for `println!`-style diagnostic output during
development and hardware bring-up, gated behind a `debug-log` Cargo feature
that is **off by default** and not enabled in the release/production build
profile.

**Rationale:** Lightweight, works with USB-Serial-JTAG out of the box, no RTT
probe required.

**Important caveat — shares the wire with the telemetry protocol:**
`esp-println`'s default output channel on the ESP32-C3 *is* the same native
USB-Serial-JTAG CDC-ACM stream that the JSON telemetry protocol is transmitted
over (Decision 6). Any `println!`/`esp_println::println!` call made while the
board is connected to the live `fermenter/` host injects a non-JSON line into
the same stream the host's line-based `Reading` parser reads, which surfaces
as a deserialize error on the host side, not on the device. "Switch to silent
operation once stable" is therefore not just a style preference — it must be
enforced by construction: all diagnostic print call sites are gated behind the
`debug-log` feature (`#[cfg(feature = "debug-log")]` or an equivalent macro
wrapper), so a normal `cargo build --release` without that feature can never
emit a stray line onto the protocol stream. Enable `debug-log` only for
bring-up sessions where the board is *not* simultaneously being read by
`fermenter/`.

**Confirmed by implementation (task 10.1):** `esp-println` cannot actually be
made an `optional`, `debug-log`-gated Cargo *dependency* — `esp-backtrace`'s
`println` feature (Decision 8) already pulls `esp-println` in unconditionally
for its own panic output, and Cargo's feature unification means our crate
must supply that shared instance a transport feature
(`esp-println` requires exactly one of `jtag-serial`/`uart`/`auto`/`no-op`)
regardless of whether `debug-log` is active. So `esp-println` is a plain
dependency with `default-features = false, features = ["jtag-serial"]`;
`debug-log` only gates *our own* `esp_println::println!` call sites, not
whether the crate is compiled in. This doesn't weaken the guarantee above —
`esp-backtrace`'s panic output only fires on an unrecoverable crash, which
isn't routine operation and isn't gated.

### Decision 8 — Required `no_std` boilerplate: panic handler and bootloader app descriptor

**Choice:** Add `esp-backtrace` (with the panic-handler feature, imported as
`use esp_backtrace as _;`) and `esp-bootloader-esp-idf` (invoking
`esp_bootloader_esp_idf::esp_app_desc!();` once at crate root) as direct
dependencies of `firmware/esp32c3`.

**Rationale:** Both are easy to omit because neither is implied by the
functional requirements above, and each fails differently:
- **No panic handler** → `firmware/esp32c3` simply does not compile
  (`#![no_std]` binaries have no default `#[panic_handler]`). This is a
  compile-time failure, at least caught by CI (task 18.2).
- **No app descriptor** → the crate compiles and flashes successfully, but the
  board never boots. Since `espflash` v4, the prebuilt esp-idf bootloader
  requires a small metadata block that `esp_app_desc!()` embeds; without it
  the failure is silent at the CI/compile stage and only surfaces as an
  unexplained "flashed but nothing happens" during hardware bring-up (task
  10.2) — exactly the kind of gap a compile-only CI job cannot catch.

Every current official `esp-hal` example includes both
(`use esp_backtrace as _;` and `esp_bootloader_esp_idf::esp_app_desc!();`),
confirming these are standard boilerplate, not project-specific choices.

**Additional toolchain note:** `esp-backtrace` on RISC-V targets needs
`-C force-frame-pointers` set in `firmware/esp32c3/.cargo/config.toml`'s
`rustflags` to produce a walkable backtrace on panic; without it, panic output
is present but useless for diagnosing a crash during bring-up.

**Confirmed by implementation (task 10.1):** the `riscv32imc-unknown-none-elf`
target installs via a plain `rustup target add riscv32imc-unknown-none-elf` —
no `espup` needed. `espup` is only required for Espressif's older Xtensa
chips (original ESP32, ESP32-S2/S3), which need a patched LLVM fork; the
RISC-V chips this project targets (ESP32-C3) use the upstream Rust compiler
and target directly, consistent with Decision 6's stable-Rust claim.
`firmware/esp32c3` was compiled clean (`cargo check`/`cargo clippy`, debug
and release, with and without `debug-log`) against real `esp-hal 1.1.1` and
`esp-rtos 0.3.0` in this environment.

### Decision 9 — Ported tuning constants: target range, EMA windows, startup default

**Choice:** Carry over the following constants verbatim from
`arduino/TempController/TempController.ino` and `ControllerActionRules.cpp`,
as named constants in `firmware/logic` (the two `ControllerActionRules`
constructor arguments) and `firmware/esp32c3` (the two `TemperatureReadings`
call sites):

| Constant | Value | Source |
|---|---|---|
| `TARGET_RANGE` | `0.3` °C | `ControllerActionRules(target, 0.3)` — defines target range (`target ± 0.3`) and failsafe (`target ± 0.6`) |
| `COOLING_OVERRUN_ADJUSTMENT` | `0.2` °C | already captured in `specs/firmware-logic/spec.md` |
| `DEFAULT_TARGET_TEMP` | `20.0` °C | firmware's own startup default, overwritten almost immediately once the host writes its persisted/configured target |
| `FERMENTER_EMA_WINDOW` | `60` | `TemperatureReadings fermenterTemperatureReadings(60)` |
| `AMBIENT_EMA_WINDOW` | `10` | `TemperatureReadings ambientTemperatureReadings(10)` |

**Rationale:** `TARGET_RANGE` is the single most load-bearing tuning number in
the whole control loop — it defines both the target band and, doubled, the
failsafe band — yet unlike `COOLING_OVERRUN_ADJUSTMENT` it was not named
anywhere in the original artifacts. Similarly, the EMA window sizes are
call-site values, not something the `TemperatureReadings` unit tests (task
8.1) can catch if wrong: the tests validate the EMA *formula*, not that
production wires up `60` for the fermenter and `10` for the ambient tracker.
Using the wrong window size wouldn't fail a test; it would just make the
fermenter reading measurably noisier or more sluggish than the original
firmware, discovered only during hardware validation (task 17) if at all.
Recording these as named constants up front (analogous to
`FERMENTER_SENSOR_ADDR` in Decision 3) closes that gap.

### Decision 10 — Sensor read failure maps to `Action::Error`

**Choice:** `SensorReader::read()` (task 13.1) returns
`Result<(f64, f64), SensorError>`, not a bare `(f64, f64)`. On `Err`, the main
loop skips `make_action_decision` for that tick and instead commands
`Action::Error` directly, which the relay layer already defines as fail-safe
(both relays LOW — see `specs/firmware-device/spec.md`, "Error action disables
both relays").

**Rationale:** The Arduino's `DallasTemperature::getTempCByIndex()` returns a
sentinel float (`-127.0`) on a disconnected or failed sensor, and the original
C++ never checks for it — a latent bug, not a behavior worth porting
faithfully. The Rust OneWire/DS18B20 crate (`onecable`, Decision 12) instead
returns a `Result`, forcing an explicit choice at exactly the point the
Arduino never had to make one. Without this decision, a read failure would either panic
(if unwrapped) or require some other ad hoc handling invented during
implementation. `Action::Error` already exists in the `Action` enum (task 2.2)
and its fail-safe relay behavior is already specified — this decision is what's
missing to actually produce it from a real failure, rather than leaving it as
an enum variant nothing ever constructs.

### Decision 11 — Persistent RX frame accumulator across ticks

**Choice:** `parse_target_frame(buf: &[u8]) -> Option<f64>` (task 14) stays a
pure, stateless function over one buffer for unit-testing purposes, but the
device-level integration (task 14.3, 16.1) wraps it with a small persistent
`heapless::Vec<u8, 16>` (or equivalent fixed-size) receive accumulator that
carries partial bytes across loop ticks, mirroring the Arduino's
`recvInProgress`/`ndx` static locals in `readSerialWithStartEndMarkers()`.

**Rationale:** The Arduino's frame reader is genuinely stateful across
`loop()` iterations, so a `<19.5>` frame whose bytes straddle a 1-second tick
boundary is never lost. As specified, `parse_target_frame` alone has no way to
retain unconsumed bytes between calls — without an explicit accumulator
wrapping it, a split frame would be silently dropped, a functional regression
against the firmware being replaced. The buffer only needs to be sized for the
longest valid frame (Arduino uses 12 bytes for `<±XXX.X>`-shaped input; 16
bytes gives headroom without meaningfully changing RAM budget).

### Decision 12 — OneWire/DS18B20 driver: `onecable`, not `one-wire-bus`/`ds18b20`

**Choice:** Use the `onecable` crate (its `OneWire` bus primitives and
`ds18b20::DS18B20` type) instead of the originally proposed `one-wire-bus` +
`ds18b20` crates.

**Rationale:** Both `one-wire-bus` and `ds18b20` (same author, both `0.1.1`,
each with a single release years ago) depend on `embedded-hal 0.2.3`'s
`digital::v2::{InputPin, OutputPin}` and `blocking::delay::DelayUs<u16>`
traits. `esp-hal 1.1.1`'s GPIO types only implement `embedded-hal 1.0`'s
traits — there is no `embedded-hal 0.2` compatibility layer in `esp-hal`
itself, so those two crates cannot be used without an extra compatibility
shim (e.g. `embedded-hal-compat`), discovered only once actually trying to
write task 11's discovery example against them (`cargo check`/`clippy`, which
don't need real peripheral types until code is written against them, hadn't
exercised this path yet).

`onecable` (`0.2.0`, actively maintained) depends on `embedded-hal 1.0`
directly — `InputPin + OutputPin + DelayNs` — matching `esp-hal 1.1.1`
natively with no shim. It is also purpose-built for DS18B20 rather than a
generic 1-Wire bus: `OneWire::search_rom_iter` discovers every ROM code on the
bus (task 11), `ds18b20::DS18B20::try_from(rom_code)` checks the family code
(`0x28`, matching the Arduino's implicit assumption), and
`DS18B20::read_temperature` does the convert-then-wait-then-read-scratchpad
sequence in one call.

**Confirmed so far (task 11):** swapping it in for `one-wire-bus`/`ds18b20`
compiles clean (`cargo check`/`clippy`, debug and release, linking
successfully). Actual bus discovery against the test board's real sensor is
the rest of task 11. No other design decision changes — Decision 3 (ROM
addressing), Decision 10 (sensor read failure → `Action::Error`), and the
OneWire wiring in `HARDWARE.md` are all crate-agnostic and unaffected.

## Risks / Trade-offs

**[ROM address discovery is a manual step]** → The `FERMENTER_SENSOR_ADDR` and
`AMBIENT_SENSOR_ADDR` constants must be determined by running a discovery helper
on the actual hardware before the production binary can be compiled. Mitigation:
document the discovery procedure in `firmware/README.md`; provide a
`sensor-discovery` binary or example in `firmware/esp32c3/` that prints ROM
addresses and exits.

**[OneWire timing on ESP32-C3]** → `onecable` (Decision 12) relies on
`embedded-hal`'s `DelayNs` for microsecond- and millisecond-scale bit timing;
timing correctness on the ESP32-C3 depends on `esp-hal`'s delay implementation
matching the DS18B20's protocol requirements. Mitigation: covered by hardware
integration test (board + real sensors); flag as the first thing to verify
during bring-up.

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
  GPIO3, and GPIO10.
  `ONE_WIRE_PIN` is now **resolved as GPIO4**: the test board's single DS18B20
  was already wired there before pin assignment was finalized, and GPIO4 (JTAG
  TMS, not a strapping pin) is safe to use since this project has no JTAG
  debugger attached — so the constant was set to match the physical build
  rather than requiring a rewire. `HEAT_RELAY_PIN`/`COOL_RELAY_PIN` remain
  open, proposed as GPIO0/GPIO1, pending relays being wired to the test board.
  _(To be resolved in task 12.1 of tasks.md — GPIO pin assignment and relay control.)_
- The `esp-rtos` runtime (Decision 2) claims `TIMG0` and the `FROM_CPU0`
  software interrupt at startup via `esp_rtos::start(...)`. Neither is a
  numbered GPIO pin, so this doesn't compete with the sensor/relay pin
  candidates above, but it does mean `TIMG0` and `FROM_CPU0` are unavailable
  for any other purpose (e.g. a second timer-driven feature added later would
  need `TIMG1` or a `SYSTIMER` alarm instead).
