# firmware/

Two-crate Cargo workspace replacing the Arduino Uno controller firmware with
Rust. See `openspec/changes/rust-firmware-esp32c3/design.md` for the full
rationale.

- `logic/` — pure, hardware-free controller logic (decision rules, EMA, relay
  action model). Builds and tests with the standard Rust toolchain, no
  embedded target required.
- `esp32c3/` — the `no_std` Embassy binary that runs on the ESP32-C3 SuperMini
  board. Requires the embedded toolchain below.

## Running the logic crate's tests

No embedded toolchain needed — this is a normal `cargo test` run:

```bash
cd firmware/logic
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Embedded toolchain setup (for `firmware/esp32c3`)

1. Install `espup` (installs the RISC-V Rust toolchain components for
   Espressif targets) and run `espup install`, then source the environment
   file it generates (`. $HOME/export-esp.sh` or per `espup`'s instructions).
2. Install `espflash` (`cargo install espflash`) — used both to flash the
   board and, via `firmware/esp32c3/.cargo/config.toml`'s `runner`, as the
   target for `cargo run`.
3. Build/flash from `firmware/esp32c3/`:

   ```bash
   cd firmware/esp32c3
   cargo build --release          # compile only
   cargo run --release            # flash + open serial monitor (espflash runner)
   espflash flash --release       # flash only, no monitor
   espflash monitor                # monitor only, after a separate flash
   ```

Note: `esp-hal` v1.x runs on **stable** Rust — `espup` installs the RISC-V
target components, not a nightly compiler. See design.md Decision 6 for the
distinction between the stable Rust toolchain and `esp-hal`'s internal
`unstable` *cargo feature*.

## The `debug-log` feature

`firmware/esp32c3` gates all `esp-println` diagnostic output behind a
`debug-log` Cargo feature, off by default:

```bash
cargo run --release --features debug-log
```

**`debug-log` must stay off whenever the board is connected to a live
`fermenter/` host.** `esp-println`'s output channel is the same
USB-Serial-JTAG stream the JSON telemetry protocol runs over — the host's
line-based `Reading` parser cannot distinguish a diagnostic line from a
telemetry line, so any `println!` while connected surfaces as a deserialize
error on the host side. Only enable it for standalone bring-up sessions
(serial echo test, sensor discovery, relay test) where nothing else is
reading the port. See design.md Decision 7.

## Sensor ROM address discovery

The two DS18B20 sensors are addressed by their unique 64-bit ROM address, not
bus-scan position (design.md Decision 3) — this must be done once per board
before the production binary can be compiled:

1. Wire the DS18B20 sensor(s) to `ONE_WIRE_PIN` (GPIO4 — see `HARDWARE.md`) in
   external-power (3-wire) mode with a 4.7 kΩ pull-up between DATA and 3V3.
2. Flash the discovery example and watch its output:

   ```bash
   cd firmware/esp32c3
   cargo run --release --example discover_sensors    # flashes + opens a monitor
   ```

   (`esp_println` output here isn't gated behind `debug-log` — this example
   is only ever run standalone, never while `fermenter/` is also reading the
   port, so Decision 7's concern doesn't apply.)

3. It prints the ROM address and family code of every DS18B20 found on the
   bus, then halts (loops idle). Record which physical sensor (fermenter or
   ambient) each address belongs to, and set `FERMENTER_SENSOR_ADDR` /
   `AMBIENT_SENSOR_ADDR` in `firmware/esp32c3/src/sensors.rs` accordingly
   before building the production binary.

**Production board status (2026-08-01):** both DS18B20s wired to GPIO4 on the
production board (shared bus, single 4.7kΩ pull-up), discovered via
`examples/identify_sensors.rs` — `rom=820000073792F228` and
`rom=52000009B5F2F728`, both valid CRC / family 0x28. Physically identified
by warming the sensor wired near the fermenter vessel and watching which
ROM's live reading rose: `820000073792F228` is `FERMENTER_SENSOR_ADDR`,
`52000009B5F2F728` is `AMBIENT_SENSOR_ADDR` — both now recorded in
`sensors.rs`.

`examples/identify_sensors.rs` is a variant of the discovery example for
when more than one sensor is wired: it scans the bus, then loops printing
each found ROM's live temperature once per second, so a sensor can be
identified by warming it and watching which address's reading responds.

Note: `OneWire::initialize_bus` (in the `onecable` crate) waits for the bus
to float high before sending a reset pulse, with no timeout — if the bus is
stuck low (missing/miswired shared pull-up, a DATA/GND swap on one sensor),
discovery hangs indefinitely rather than erroring. If a discovery/identify
run prints nothing past its initial "Scanning..." line, check the wiring
before assuming a code problem.

## Hardware integration test

`firmware/logic/tests/hardware.rs` opens a real serial port and confirms the
attached board (flashed with the production `esp32c3` binary, `debug-log`
off) emits a JSON line that parses as a valid `Reading`. `#[ignore]`'d —
never runs in CI or under plain `cargo test`:

```bash
cd firmware/logic
cargo test -- --ignored           # SERIAL_PORT/SERIAL_BAUD env vars override the /dev/ttyACM0 @ 115200 default
```

Lives in `logic/`, not `esp32c3/` — see the file's own doc comment for why
(`esp32c3`'s `.cargo/config.toml` pins it to the embedded target, which
can't host a real-serial-port test; `logic` already runs its tests on the
host with no embedded toolchain).
