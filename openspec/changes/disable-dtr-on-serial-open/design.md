## Context

`ArduinoSerialSource::ensure_connected` (`fermenter/src/serial/arduino.rs`)
opens the port with `tokio_serial::new(&self.port_path, self.baud_rate).open_native_async()`
and no further builder configuration. `tokio-serial` (via the underlying
`serialport` crate) asserts DTR and RTS by default when a port is opened. On
genuine Arduino Uno/Nano/Mega boards, the USB-serial chip's DTR line is
coupled through a capacitor to the ATmega reset pin — asserting DTR triggers
a hardware reset of the sketch. This is a well-known Arduino IDE behavior
(it's how the IDE auto-resets the board before uploading), not a bug in
`tokio-serial`.

`ArduinoSerialSource` reopens the port (calling `ensure_connected` again)
whenever `self.stream` is set to `None`: after EOF, a read error, a write
error, or a flush error — see `arduino.rs:101,120,148,156`. Each of these is
a legitimate, expected occurrence in long-running USB-serial operation, not
just at process startup. Every reopen therefore silently reboots the
attached Arduino, resetting the firmware's monotonic min/max temperature
tracking (`TemperatureReadings::clear()` is only called in the constructor —
`arduino/TempController/TemperatureReadings.cpp:6-8`) and briefly halting
temperature control while the sketch re-runs `setup()`.

This was diagnosed from a real symptom: the dashboard's `min` reading showed
`14.0` while the chart (sourced independently from Redis time-series history)
showed the fermenter average dipping to `13.6` within the same 6-hour window
— consistent with an Arduino reset partway through that window that the user
did not otherwise observe (Pi/Arduino share a power source with 24h+ uptime).

## Goals / Non-Goals

**Goals:**
- Stop the Arduino from resetting every time the Rust app opens or reopens
  the serial port, so its lifetime min/max readings (and any in-progress
  control state) survive transient reconnects.
- Prove the fix works against real hardware, since DTR/reset behavior cannot
  be exercised through the mock `SerialSource`.

**Non-Goals:**
- Changing the serial framing contract, baud rate, or JSON schema — all
  fixed per `device-connection`'s existing "Serial framing and baud contract"
  requirement.
- Adding a rolling/windowed min-max to the firmware (a legitimate follow-up,
  but out of scope here — this change only fixes spurious resets, not the
  firmware's all-time-since-boot semantics).
- Changing reconnect/backoff behavior itself (slice-6's exponential backoff
  is unaffected; only the DTR side-effect of opening the port changes).

## Decisions

1. **Disable DTR and RTS on the builder, not via a post-open ioctl.**
   `tokio_serial::new(...)` returns a `serialport::SerialPortBuilder`, which
   exposes `.set_dtr(bool)` and `.set_rts(bool)` as chainable builder
   methods applied at open time — no separate call after
   `open_native_async()` is needed. This keeps the change to a single
   builder-chain edit in `ensure_connected`.
   - *Alternative considered:* call `.write_data_terminal_ready(false)` on
     the opened `SerialStream` after `open_native_async()` succeeds. Rejected
     because the reset already fires during the open syscall itself (the port
     configuration, including DTR/RTS, is applied as part of opening the
     underlying fd) — configuring it post-open would be too late to prevent
     the reset on that particular open.

2. **Disable RTS as well as DTR.** Some USB-serial adapters/boards use RTS
   (or a DTR+RTS combination gated through control logic on the adapter) to
   drive reset, depending on the board and bootloader. Disabling both is the
   standard recommendation (matches Arduino IDE's own "disable hardware
   reset" workarounds) and has no downside for this project since the
   firmware doesn't rely on hardware flow control.

3. **Prove it with a new hardware-only test, not a unit test.** DTR/reset
   behavior only manifests against a real USB-serial-to-Arduino link; the
   `MockSerialSource` has no serial hardware underneath it to reset. Per the
   existing convention (`tests/serial_hardware.rs`, `#[ignore]`'d,
   `cargo test -- --ignored`), a new test opens a first connection, drops
   it, opens a second connection to the same port, and asserts a line
   arrives within a window tight enough that a DTR-triggered reset (~2s
   setup delay stacked onto the firmware's ~10s print cycle, i.e. ~12-13s)
   would miss it. No sensors are required — the test only proves timing, not
   temperature values, matching the existing hardware tests' approach.

## Risks / Trade-offs

- **[Risk] Some USB-serial adapters may not support disabling DTR/RTS, or
  may behave differently.** → `serialport`'s `set_dtr`/`set_rts` are
  supported broadly across the CDC-ACM and common USB-serial chips (FTDI,
  CH340, CP210x) used with Arduino boards; if an unusual adapter doesn't
  honor it, behavior degrades to today's status quo (occasional reset), not
  a regression.
- **[Risk] The new hardware test is timing-based, not a direct assertion of
  "no reset happened."** → Acceptable per user confirmation: no temperature
  sensors are attached to the dev/test Arduino, so timing is the only
  externally observable proxy available without sensors. Full confidence
  comes from the documented manual verification step on the real Pi/Arduino
  deployment.
</content>
