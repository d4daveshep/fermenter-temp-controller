## Why

The dashboard's `min`/`max` fields are the Arduino's running min/max since its
last reset (`TemperatureReadings::clear()` is only called at boot). The Rust
app drops the serial stream on every transient read/write/flush error,
causing a port reopen. On Linux the kernel asserts DTR during the `open()`
syscall (USB CDC ACM protocol — unavoidable from userspace), and on a
genuine Arduino DTR is wired through a capacitor to the reset pin, so each
reopen silently reboots the sketch. The reboot clears the running min/max, so
the dashboard can show a `min` that's higher than temperatures visible in the
same time window on the chart (which reads full history from Redis, unaffected
by Arduino resets). This was observed directly: dashboard `min=14.0` while the
chart showed the fermenter average dipping to `13.6` within the same 6-hour
window, despite the Pi (and thus the Arduino, powered from the same source)
never having been power-cycled.

## What Changes

- Keep the serial stream alive on transient read/write/flush errors instead
  of dropping it. The port is only closed on actual EOF (device disconnected).
  This is the primary fix — by never reopening on transient errors, we never
  trigger the kernel's DTR reset pulse on the Arduino.
- Add `.dtr_on_open(false)` to the serial port builder as a secondary,
  best-effort measure to clear DTR after open on platforms that support it.
- Add a hardware-in-the-loop regression test proving two consecutive reads
  on the same source both succeed (the stream was not closed between them).

## Capabilities

### New Capabilities

<!-- None: no new capability introduced. -->

### Modified Capabilities

- `device-connection`: The serial transport SHALL keep the serial stream open
  across transient read/write/flush errors, only closing it on EOF, so that
  the attached Arduino is not reset by a DTR-triggered port reopen during
  normal operation. The port SHALL also be opened with `.dtr_on_open(false)`
  as a best-effort measure to clear DTR after open.

## Impact

- **Code:** `fermenter/src/serial/arduino.rs` — remove `self.stream = None`
  from read/write/flush error handlers; add `.dtr_on_open(false)` to the
  `tokio_serial::new(...)` builder chain in `ensure_connected()`.
- **Tests:** `fermenter/tests/serial_hardware.rs` — new `#[ignore]`'d
  `consecutive_reads_on_same_source` test (hardware-only, per existing
  convention).
- No changes to the firmware, serial framing contract, model, storage, or web
  layers.
