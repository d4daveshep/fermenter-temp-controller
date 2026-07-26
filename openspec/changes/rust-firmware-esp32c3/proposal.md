## Why

The fermenter controller currently runs C++ firmware on an Arduino Uno, a board
that cannot run Rust. Replacing it with an Espressif ESP32-C3 Mini DevKit
(native USB-Serial-JTAG, RISC-V, first-class Rust support via `esp-hal` +
Embassy) lets the entire system — Pi host and controller board alike — be
written in Rust, tested with the same toolchain, and maintained under the same
OpenSpec capability library. The LCD has also been removed from the hardware
build, further simplifying the firmware surface.

## What Changes

- **New `firmware/` Cargo workspace** added at repo root, containing two crates:
  - `firmware/logic/` — pure, `no_std`-compatible Rust port of the controller
    logic (temperature decision rules, EMA readings, relay action model). No
    hardware dependencies; runs under plain `cargo test` on any host.
  - `firmware/esp32c3/` — `no_std` Embassy binary crate targeting
    `riscv32imc-unknown-none-elf` (ESP32-C3). Depends on `logic/`; drives the
    two relay GPIO outputs, reads two DS18B20 temperature sensors over a shared
    OneWire bus, and implements the serial telemetry protocol over native
    USB-Serial-JTAG.
- **Two behavioral improvements over the Arduino firmware** (documented quirks,
  not surprises):
  - **Zero-target fix**: the original C++ treats a parsed `0.0` as "no frame
    received"; the Rust firmware will distinguish a missing frame from a valid
    `0.0` target, allowing lagering temperatures.
  - **ROM-address sensor identification**: sensors are addressed by their unique
    64-bit ROM addresses rather than bus-scan index, removing the silent
    fragility around sensor replacement/rewiring.
- **No change to the serial contract**: 115200 baud, newline-delimited JSON
  readings out, `<float>`-framed target in — identical to what `fermenter/`
  currently consumes. `fermenter/` requires no code changes.
- **LCD removed**: `updateLCD()` and the 6 LCD GPIO pins are dropped; the new
  board uses native USB-Serial-JTAG, so no separate UART bridge chip occupies
  GPIO.
- **`arduino/TempController/`** stays in the repo untouched as historical
  reference.

## Capabilities

### New Capabilities

- `firmware-logic`: The pure, hardware-free controller logic — temperature
  decision rules (RC1–RC10), exponential moving average, min/max tracking, and
  the `Action`/`Decision` types. Fully unit-tested without embedded toolchain.
- `firmware-device`: The ESP32-C3 binary — Embassy task loop, DS18B20 OneWire
  sensor reads (ROM-addressed), relay GPIO control, USB-Serial-JTAG protocol
  implementation (JSON telemetry out, `<target>` frame parsing in).

### Modified Capabilities

- `device-connection`: The existing spec describes the serial contract from the
  host side (baud rate, JSON framing, `SerialSource` abstraction). It currently
  refers to "the Arduino" as the concrete device. The spec requirements are
  unchanged — the wire contract is identical — but the language needs updating
  to be device-agnostic (the concrete implementation moves from Arduino/C++ to
  ESP32-C3/Rust), and two behavioral improvements (zero-target fix, ROM-address
  sensor identification) need documenting.

## Impact

- **`firmware/`** — new directory, new Cargo workspace; not a member of
  `fermenter/`'s Cargo workspace (different target triple and toolchain).
- **`fermenter/`** — no code changes; only the env-var `SERIAL_PORT` may need
  updating if the ESP32-C3 enumerates at a different `/dev/tty*` path than the
  Arduino did (typically `/dev/ttyACM0` for both native USB-CDC on Linux, but
  worth verifying on the Pi).
- **`arduino/TempController/`** — preserved unchanged as historical reference.
- **Dependencies** (firmware only): `esp-hal`, `esp-rtos` (`embassy` feature —
  provides the Embassy executor/time driver and the `#[esp_rtos::main]` entry
  point; successor to the now-stale `esp-hal-embassy`), `embassy-executor`,
  `embassy-time`, `embedded-hal`, `embedded-io-async` (the `Read`/`Write`
  traits `UsbSerialJtagRx`/`Tx` implement — needed explicitly since our code
  calls them via the trait, not an inherent method), `onecable` (OneWire bus +
  DS18B20 support — not the originally proposed `one-wire-bus`/`ds18b20`,
  which turned out to require `embedded-hal 0.2` with no `esp-hal`
  compatibility path; see design.md Decision 12), `serde-json-core`,
  `heapless`, `esp-println` (kept off the
  telemetry stream by gating our own call sites behind a `debug-log` feature
  — see design.md Decision 7; the crate itself is pulled in unconditionally
  by `esp-backtrace`'s panic-print support regardless of that feature),
  `esp-backtrace` (panic handler — required for the crate to compile at all)
  and `esp-bootloader-esp-idf` (`esp_app_desc!()` — required by `espflash`
  v4's bootloader for the board to boot; see design.md Decision 8); all
  `no_std`-compatible. All confirmed by actually compiling
  `firmware/esp32c3` against real `esp-hal 1.1.1`/`esp-rtos 0.3.0` (the
  `riscv32imc-unknown-none-elf` target installs via plain `rustup target add`
  — no `espup` needed for RISC-V chips, only for the older Xtensa ones).
- **Tooling**: `espflash` replaces `compile_arduino.sh`/`upload_arduino.sh` for
  build and flash.
