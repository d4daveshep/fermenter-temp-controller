# Tasks: rust-firmware-esp32c3

## 1. Workspace scaffold

- [x] 1.1 Create `firmware/` directory with a Cargo workspace `Cargo.toml` declaring members `["logic", "esp32c3"]`
- [x] 1.2 Create `firmware/logic/Cargo.toml` as a `no_std`-compatible library crate; add `rstest` as a dev-dependency
- [x] 1.3 Create `firmware/esp32c3/Cargo.toml` targeting `riscv32imc-unknown-none-elf`; declare `logic` as a path dependency; add `esp-hal`, `esp-rtos` (with the `embassy` feature enabled), `embassy-executor`, `embassy-time`, `embedded-hal`, `embedded-io-async`, `onecable` (OneWire/DS18B20 — swapped in for the originally proposed `one-wire-bus`/`ds18b20`, which need `embedded-hal 0.2` with no `esp-hal 1.1` compatibility path; see design.md Decision 12), `serde-json-core`, `heapless`, `esp-backtrace` (panic handler), `esp-bootloader-esp-idf` (app descriptor). `esp-println` is a plain (non-optional) dependency with `default-features = false, features = ["jtag-serial"]`: it turns out `esp-backtrace`'s `println` feature already pulls `esp-println` in unconditionally for panic output and needs a transport feature selected regardless of our own gate, so making our own reference to it `optional` bought nothing — the `debug-log` feature (off by default — see design.md Decision 7) instead just gates our own `esp_println::println!` call sites behind `#[cfg(feature = "debug-log")]`. Verified by actually compiling against real `esp-hal 1.1.1`/`esp-rtos 0.3.0` (`rustup target add riscv32imc-unknown-none-elf` — no `espup` needed for this RISC-V target).
- [x] 1.4 Create `firmware/esp32c3/.cargo/config.toml` with target triple, `espflash flash --monitor` as the runner, and `rustflags` containing both `-C link-arg=-Tlinkall.x` (pulls in esp-hal's generated linker script — required, or linking fails with undefined-symbol errors for peripheral/interrupt-vector symbols like `GPIO`, `WIFI_MAC`, `__EXTERNAL_INTERRUPTS`) and `-C force-frame-pointers` (needed for usable `esp-backtrace` panic output on RISC-V). Confirmed by an actual `cargo build --release` + link + flash + echo round trip in task 10.2.
- [x] 1.5 Create `firmware/README.md` documenting toolchain setup (`espup`, `espflash`), how to run `cargo test` for the logic crate, the sensor ROM address discovery procedure, and the `debug-log` feature (what it does, and that it must stay off when connected to a live `fermenter/` host — see design.md Decision 7)

## 2. `firmware-logic`: Action and Decision types (TDD)

- [x] 2.1 Write failing tests for `Action` enum variants and `Decision` struct: `stores_action_and_reason`, `action_text_for_each_variant`, `is_made_returns_false_before_set`, `is_made_returns_true_after_set`, `clear_resets_to_unmade`
- [x] 2.2 Implement `Action` enum (`Rest`, `Heat`, `Cool`, `Error`, sentinel) and `Decision` struct (`action`, `reason_code`, `action_text()`, `is_made()`, `clear()`) in `firmware/logic/src/decision.rs` to make the tests pass

## 3. `firmware-logic`: Natural drift classification (TDD)

- [x] 3.1 Write failing tests for `NaturalDrift` classification: ambient equals fermenter → `NaturalHeating`, ambient above fermenter → `NaturalHeating`, ambient below fermenter → `NaturalCooling`
- [x] 3.2 Implement `NaturalDrift` enum and `get_natural_drift(ambient: f64, actual: f64) -> NaturalDrift` in `firmware/logic/src/controller_action_rules.rs` to make the tests pass

## 4. `firmware-logic`: Failsafe boundary enforcement (TDD)

- [x] 4.1 Write failing tests for failsafe: all 6 combinations of `(REST|HEAT|COOL) × (ambientLow|ambientHigh)` when temp is below failsafe → `Heat`/`"RC1"`; same matrix when temp is above failsafe → `Cool`/`"RC5"` (porting the assertions from `Test_ControllerActionRules.cpp` tests 1, 5, 6, 10)
- [x] 4.2 Implement `check_failsafe_min` and `check_failsafe_max` in `ControllerActionRules` to make the tests pass, using `TARGET_RANGE = 0.3` (failsafe is `target ± 2 × TARGET_RANGE`, i.e. `± 0.6`) per design.md Decision 9

## 5. `firmware-logic`: Cooling overrun adjustment (TDD)

- [x] 5.1 Write failing tests for cooling overrun: `COOL + temp < stop_cooling_temp + NaturalHeating → Rest/"RC2.2"` and `COOL + temp < stop_cooling_temp + NaturalCooling → Heat/"RC7.2"` (porting `AdjustmentForCoolingOverrun` and the RC2.2/RC7.2 cases in `WhatToDoNext`)
- [x] 5.2 Implement `check_cooling_overrun` and `get_stop_cooling_temp` in `ControllerActionRules` to make the tests pass

## 6. `firmware-logic`: Temperature range decision rules (TDD)

- [x] 6.1 Write failing tests for below-range decisions: all four cases RC2.1, RC2.3, RC7.1, RC7.3 (porting the RC2 and RC7 assertions from `WhatToDoNext`)
- [x] 6.2 Implement `decide_when_below_range` to make the tests pass
- [x] 6.3 Write failing tests for in-range decisions: all six cases RC3.1, RC3.2, RC3.3, RC8.1, RC8.2, RC8.3
- [x] 6.4 Implement `decide_when_in_range` to make the tests pass
- [x] 6.5 Write failing tests for above-range decisions: all six cases RC4.1, RC4.2, RC4.3, RC9.1, RC9.2, RC9.3
- [x] 6.6 Implement `decide_when_above_range` to make the tests pass

## 7. `firmware-logic`: Target temperature mutability (TDD)

- [x] 7.1 Write failing test: `set_target_temp` persists and is returned by `get_target_temp`; subsequent `make_action_decision` uses the new value (porting `UpdatedTargetTempIsSaved`)
- [x] 7.2 Implement `set_target_temp` / `get_target_temp` and wire the top-level `make_action_decision` method that orchestrates all the sub-checks, to make the tests pass; construct `ControllerActionRules` with `DEFAULT_TARGET_TEMP = 20.0` and `TARGET_RANGE = 0.3` per design.md Decision 9 / `HARDWARE.md`

## 8. `firmware-logic`: Exponential moving average (TDD)

- [x] 8.1 Write failing tests for `TemperatureReadings`: `seed_sets_initial_average`, `seed_ignored_after_first_reading`, `ema_matches_expected_value_over_known_sequence`, `min_and_max_tracked_correctly`, `count_increments_with_each_update` (porting the AUnit `TemperatureReadingsTest` suite with the same known random seed and expected EMA value `12.14163`)
- [x] 8.2 Implement `TemperatureReadings` struct with `set_initial_average`, `update`, `average`, `minimum`, `maximum`, `count` in `firmware/logic/src/temperature_readings.rs` to make the tests pass

## 9. `firmware-logic`: Verify complete logic test coverage

- [x] 9.1 Run `cargo test` in `firmware/logic/` — all tests green, no warnings
- [x] 9.2 Run `cargo clippy -- -D warnings` in `firmware/logic/` — clean
- [x] 9.3 Run `cargo fmt --check` in `firmware/logic/` — clean

## 10. `firmware-device`: Hardware bring-up — serial echo

- [x] 10.1 Create `firmware/esp32c3/src/main.rs` with a `#[esp_rtos::main]` async entry point; call `esp_rtos::start(timg0.timer0, sw_int.software_interrupt0)` before spawning any task (see design.md Decision 2); add a USB-Serial-JTAG echo task using `esp-hal`'s `UsbSerialJtag` peripheral
- [x] 10.2 Flash to the test board and verify the echo works over `/dev/ttyACM0` — confirms toolchain, flash, and USB-Serial-JTAG are all functional. Done via `cargo build --release` + `espflash flash --non-interactive` (not `cargo run`'s configured runner directly — that runner appends `--monitor`, which blocks forever and isn't scriptable; use `cargo run --release` interactively instead when at a real terminal). Verified with a small pyserial script sending three round-trip messages (plain text, an RC-code-shaped string, and a `<19.5>`-framed value) — all three echoed back byte-for-byte. `firmware/esp32c3/.cargo/config.toml` was missing the linker-script rustflag (`-C link-arg=-Tlinkall.x`) needed to resolve esp-hal's peripheral/interrupt-vector symbols — added, confirmed by an actual successful link + flash + echo.

## 11. `firmware-device`: Sensor ROM address discovery

- [x] 11.1 Create `firmware/esp32c3/examples/discover_sensors.rs` that scans the OneWire bus and prints the ROM address of every DS18B20 found, then halts. Uses `onecable::OneWire::search_rom_iter` (see design.md Decision 12) against an `esp_hal::gpio::Flex` pin configured as true open-drain input+output (`DriveMode::OpenDrain`, `Pull::None` — matches the external 4.7kΩ pull-up). Verified compiling and linking for real.
- [x] 11.2 Wire the DS18B20 sensor(s) to the chosen OneWire GPIO pin (GPIO4 — see HARDWARE.md) with a 4.7kΩ pull-up; flash the discovery example and record ROM addresses. Only one sensor is currently wired to the test board; flashed and ran against real hardware, discovered `family=0x28 rom=2C0000074DC9F928` with a valid CRC. The second sensor (ambient) is not yet wired — re-run once it is.
- [x] 11.3 Add `FERMENTER_SENSOR_ADDR: [u8; 8]` and `AMBIENT_SENSOR_ADDR: [u8; 8]` compile-time constants in `firmware/esp32c3/src/sensors.rs` with the discovered values; document the discovery step in `firmware/README.md`. `FERMENTER_SENSOR_ADDR` holds the real discovered address (tentatively assigned — not yet confirmed which physical sensor this is); `AMBIENT_SENSOR_ADDR` is still a placeholder pending the second sensor.

## 12. `firmware-device`: GPIO pin assignment and relay control

- [x] 12.1 Confirm free GPIO pins on the board silkscreen/schematic; record `HEAT_RELAY_PIN`, `COOL_RELAY_PIN`, `ONE_WIRE_PIN` as named constants in `firmware/esp32c3/src/relays.rs` and `sensors.rs`, verifying none are GPIO2, GPIO8, or GPIO9. `HEAT_RELAY_PIN`/`COOL_RELAY_PIN` = GPIO0/GPIO1 (proposed, per HARDWARE.md); these are documentation/cross-reference constants — the actual peripheral selection happens via `peripherals.GPIO0`/`GPIO1`'s distinct esp-hal types, not a runtime pin-number lookup. Added a `src/lib.rs` (auto-detected by Cargo alongside `main.rs`'s `[[bin]]`) so `relays`/`sensors` modules are shared between `main.rs` and `examples/*.rs` without duplication.
- [x] 12.2 Implement `RelayController` in `firmware/esp32c3/src/relays.rs`: owns the two `Output` GPIO handles; `set(action: Action)` drives them per the spec (Rest → both LOW, Heat → heat HIGH cool LOW, Cool → heat LOW cool HIGH, Error → both LOW; `NoAction` sentinel also treated as fail-safe both-LOW, defensively). Verified compiling and linking for real.
- [ ] 12.3 Flash a relay test to the board (cycle through Rest → Heat → Cool → Rest once per second); verify with a multimeter or LED that pin logic matches spec — **deferred**: no relays or LEDs wired to `HEAT_RELAY_PIN`/`COOL_RELAY_PIN` yet. `examples/relay_test.rs` is written and verified compiling/linking (`cargo run --release --example relay_test`); run it once something is wired to GPIO0/GPIO1 to observe.

## 13. `firmware-device`: DS18B20 temperature sensor reads

- [ ] 13.1 Implement `SensorReader` in `firmware/esp32c3/src/sensors.rs`: initialises the OneWire bus, locates each sensor by its ROM address, triggers conversion, and reads back both temperatures as `Result<(f64, f64), SensorError>` (fermenter, ambient) — do not silently coerce a failed read into a bogus float; see design.md Decision 10
- [ ] 13.2 Flash a sensor-read loop to the test board; verify both temperatures are printed over USB-Serial-JTAG — confirms OneWire timing, pull-up resistor, and ROM address constants are correct

## 14. `firmware-device`: Target frame parser (TDD in `logic`, integration test on device)

- [ ] 14.1 Write failing tests in `firmware/logic/src/protocol.rs` (or a host-side test module): `valid_frame_returns_some_value`, `zero_frame_returns_some_zero` (not None), `no_frame_returns_none`, `partial_frame_returns_none`, `malformed_frame_returns_none`
- [ ] 14.2 Implement `parse_target_frame(buf: &[u8]) -> Option<f64>` in `firmware/logic/src/protocol.rs` to make the tests pass
- [ ] 14.3 Wire `parse_target_frame` into the USB-Serial-JTAG RX path in `esp32c3/src/main.rs`, wrapped in a persistent `heapless::Vec<u8, 16>` accumulator that carries unconsumed bytes across ticks so a frame split across a tick boundary is not lost (see design.md Decision 11 — mirrors the Arduino's stateful `recvInProgress`/`ndx`); flash to the test board and send `<19.5>` from a terminal — verify the target updates in the printed log

## 15. `firmware-device`: JSON telemetry output (TDD in `logic`, integration test on device)

- [ ] 15.1 Write failing tests in `firmware/logic/src/protocol.rs` for `format_telemetry`: emitted JSON contains all required keys (`target`, `average`, `min`, `max`, `ambient`, `action`, `reason-code`); `action` strings match host-expected values; `reason-code` is byte-identical to the RC code; output is newline-terminated
- [ ] 15.2 Implement `format_telemetry(readings: &TelemetryData) -> heapless::String<256>` using `serde-json-core` to make the tests pass
- [ ] 15.3 Wire `format_telemetry` into the Embassy task's 10-second emission path; flash to the test board and verify the JSON output is parseable by the `fermenter/` host's `Reading` deserialiser (use `serde_json::from_str::<Reading>` in a small CLI or REPL)

## 16. `firmware-device`: Complete main control loop

- [ ] 16.1 Assemble the full Embassy task loop in `firmware/esp32c3/src/main.rs`: 1s tick → poll serial RX for `<target>` frame → read both sensors → update EMAs → `make_action_decision` → `relay_controller.set(action)` → emit JSON every 10 ticks; seed both EMAs on startup from first sensor read; construct the fermenter `TemperatureReadings` with window `60` and the ambient one with window `10` (design.md Decision 9); if `SensorReader` returns `Err`, skip `make_action_decision` for that tick and call `relay_controller.set(Action::Error)` directly (design.md Decision 10)
- [ ] 16.2 Flash to the test board; connect to `fermenter/` host with `MOCK_SERIAL=false`; confirm readings are ingested, dashboard updates, and `POST /target` causes the board to apply the new target within one loop cycle

## 17. End-to-end hardware validation

- [ ] 17.1 Run `fermenter/` against the test board for at least 10 minutes; confirm readings, relay switching, and target reconcile all behave as expected; verify `reason-code` values appear correctly in the dashboard
- [ ] 17.2 Flash the production board with the finalised binary; swap for the Arduino Uno; repeat the 10-minute validation
- [ ] 17.3 Add a `#[test] #[ignore]` hardware integration test in `firmware/esp32c3/tests/hardware.rs` (or `firmware/tests/`) that opens the serial port, reads one JSON line, and verifies it deserialises as a valid `Reading` — to be run manually with `cargo test -- --ignored` when the board is attached

## 18. CI and tooling

- [ ] 18.1 Add a GitHub Actions job to `.github/workflows/rust.yml` that runs `cargo test` and `cargo clippy -- -D warnings` in `firmware/logic/` (host target, no embedded toolchain required)
- [ ] 18.2 Add a separate CI job (or step) that installs the RISC-V target and attempts `cargo build --release` in `firmware/esp32c3/` (compile-only, no flash) to catch regressions in the device crate
- [ ] 18.3 Confirm `arduino/TempController/compile_arduino.sh` and `upload_arduino.sh` still work (no changes made to the Arduino tree) — leave them in place as documented historical reference
