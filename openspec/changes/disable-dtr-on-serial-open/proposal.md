## Why

The dashboard's `min`/`max` fields are the Arduino's running min/max since its
last reset (`TemperatureReadings::clear()` is only called at boot). Every time
the Rust app opens the serial port — at startup, and again after any
transient read/write error causes `ArduinoSerialSource` to drop and reopen
its stream — `tokio_serial::new()` asserts DTR by default. On a genuine
Arduino, DTR is wired through a capacitor to the reset pin, so this silently
reboots the sketch. The reboot clears the running min/max, so the dashboard
can show a `min` that's higher than temperatures visible in the same time
window on the chart (which reads full history from Redis, unaffected by
Arduino resets). This was observed directly: dashboard `min=14.0` while the
chart showed the fermenter average dipping to `13.6` within the same 6-hour
window, despite the Pi (and thus the Arduino, powered from the same source)
never having been power-cycled.

## What Changes

- Configure the serial port builder in `ArduinoSerialSource::ensure_connected`
  to not assert DTR or RTS on open, so opening (or reopening, after a
  transient error) the port no longer resets the Arduino.
- Add a hardware-in-the-loop regression test proving a second connection to
  the same port does not trigger the firmware's post-reset startup delay.

## Capabilities

### New Capabilities

<!-- None: no new capability introduced. -->

### Modified Capabilities

- `device-connection`: The serial transport SHALL open the port without
  asserting DTR or RTS, so connecting or reconnecting never resets the
  attached Arduino.

## Impact

- **Code:** `fermenter/src/serial/arduino.rs` — add `.set_dtr(false)` and
  `.set_rts(false)` to the `tokio_serial::new(...)` builder chain in
  `ensure_connected()`.
- **Tests:** `fermenter/tests/serial_hardware.rs` — new `#[ignore]`'d test
  (hardware-only, per existing convention).
- No changes to the firmware, serial framing contract, model, storage, or web
  layers.
</content>
