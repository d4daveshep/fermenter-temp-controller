## Context

`ArduinoSerialSource::ensure_connected` (`fermenter/src/serial/arduino.rs`)
opens the port with `tokio_serial::new(&self.port_path, self.baud_rate).open_native_async()`
and no further builder configuration. On genuine Arduino Uno/Nano/Mega
boards, the USB-serial chip's DTR line is coupled through a capacitor to the
ATmega reset pin — asserting DTR triggers a hardware reset of the sketch.
This is a well-known Arduino IDE behavior (it's how the IDE auto-resets the
board before uploading), not a bug in `tokio-serial`.

On Linux, the kernel's USB CDC ACM driver asserts DTR as a side effect of the
`open()` syscall — this is a USB protocol requirement and cannot be prevented
from userspace. The `serialport` crate's `.dtr_on_open(false)` builder method
(applied via `TIOCMBIC` ioctl after open) cannot prevent the initial
kernel-level DTR pulse, only clear DTR afterward.

The real problem is that `ArduinoSerialSource` was reopening the port on
**every** transient error: `self.stream = None` on any read error, write
error, or flush error (see arduino.rs:144,172,180). USB serial connections
experience occasional transient noise (buffer overruns, framing errors), and
each such error prompted a reopen — which triggered a DTR-based Arduino
reset, silently clearing the firmware's lifetime min/max tracking.

This was diagnosed from a real symptom: the dashboard's `min` reading showed
`14.0` while the chart (sourced independently from Redis time-series history)
showed the fermenter average dipping to `13.6` within the same 6-hour window
— consistent with an Arduino reset partway through that window that the user
did not otherwise observe (Pi/Arduino share a power source with 24h+ uptime).

## Goals / Non-Goals

**Goals:**
- Stop the Arduino from resetting on transient serial errors (read, write,
  flush) — the primary cause of spurious min/max resets during long-running
  operation. The port should only reopen on actual disconnection (EOF).
- Add `.dtr_on_open(false)` to the serial port builder as a best-effort
  measure to clear DTR after open when the platform supports it.
- Prove the stream stays alive across reads with a hardware timing test.

**Non-Goals:**
- Preventing the DTR pulse at process startup — **not possible from
  userspace.** The Linux kernel's USB CDC ACM driver (`acm_port_activate` in
  `drivers/usb/class/cdc-acm.c`) asserts DTR unconditionally during the
  `open()` syscall. This is a USB protocol requirement that cannot be
  overridden by any `ioctl` or `tcsetattr` call. As a result, every Docker
  container restart (which kills the process and closes the fd, then starts
  a new process that opens the port anew) always resets the Arduino. A
  hardware fix (e.g. 10–100µF capacitor between the ATmega's RESET pin and
  GND to filter the DTR pulse) would be required to prevent startup
  resets, and was considered but explicitly not pursued per user decision.
- Preventing DTR resets on explicit device unplug/replug — the firmware
  legitimately needs to restart after a physical disconnect.
- Changing the serial framing contract, baud rate, or JSON schema — all
  fixed per `device-connection`'s existing "Serial framing and baud contract"
  requirement.
- Adding a rolling/windowed min-max to the firmware — out of scope.

## Decisions

### Decision 1: Keep the stream alive on transient errors (primary fix)

Instead of `self.stream = None` on read/write/flush errors, log the error
and return it to the caller while keeping the stream open. The caller
(`ingest_loop` in `ingest.rs`) already retries on error with its own delay,
so the next call will retry on the same open connection. Only EOF (the device
physically disconnected) drops the stream, triggering a full reopen.

This directly addresses the root cause: transient USB serial noise no longer
causes a port reopen, which means no DTR pulse, which means the Arduino is
never silently reset mid-session.

- *Alternative considered:* Recovering in-place after errors by flushing the
  buffer and retrying internally. Rejected because `read_line` cannot
  distinguish "noise on the wire" from "partial line mid-stream" — retrying
  internally could enter an infinite loop on a persistent hardware fault.
  Returning the error to the caller (which has its own backoff/retry logic)
  is more robust.

### Decision 2: Use `.dtr_on_open(false)` on the builder (secondary fix)

`serialport` v4.9.0's `SerialPortBuilder` has `.dtr_on_open(bool)` (not
`.set_dtr(bool)` — no such builder method exists). After the kernel's
`open()` syscall and termios setup, the builder applies `dtr_on_open` via
`TIOCMBIC`/`TIOCMBIS` ioctls. Setting it to `false` clears DTR immediately
after open.

On Linux this cannot prevent the kernel-level DTR pulse during `open()`, but
it ensures DTR is LOW after open and helps on platforms where the
`serialport` crate has full control (Windows, macOS).

- *Alternative considered:* calling `write_data_terminal_ready(false)` on the
  opened `SerialStream` after `open_native_async()` succeeds. Equivalent
  effect but requires an extra method call — the builder handles it inline.

### Decision 3: RTS builder control is unavailable in `serialport` v4.9.0

The `SerialPortBuilder` in this version has no `set_rts` or `rts_on_open`
method. RTS is left in whatever state the kernel sets it to during `open()`.
On Arduino boards the reset is driven exclusively through DTR (via the 100nF
capacitor to the ATmega reset pin), so RTS not being controllable is
acceptable.

### Decision 4: Prove it with a hardware timing test

The new test `consecutive_reads_within_print_interval` opens a single source,
reads one line (proving the connection), then reads a second line within 11s.
If the stream is kept alive (no error-triggered reopen), the second line
arrives within ~10s — the firmware's normal print interval. If the stream
was dropped, a reopen would trigger DTR reset and push the wait past 12s.

## Risks / Trade-offs

- **[Risk] A genuinely broken serial connection (device unplugged then
  replugged) generates repeated read errors without recovery until EOF is
  reached.** → The kernel eventually returns EOF after the device disappears;
  once EOF is received, the existing reconnect-with-backoff kicks in and
  recovers normally.
- **[Risk] The hardware test is timing-based.** → Acceptable per user
  confirmation: no temperature sensors are attached to the dev/test Arduino,
  so timing is the only observable proxy without sensors.
- **[Risk] `dtr_on_open(false)` may not be supported on all USB-serial
  chips.** → If unsupported, the builder call is a no-op — behavior degrades
  to the existing `dtr_on_open = None` default, which is the pre-fix
  baseline.
- **[Observed limitation] Docker container restart always resets the
  Arduino** — confirmed on hardware. The kernel's `open()` syscall on the
  USB CDC ACM device unconditionally asserts DTR. Since the container
  restart kills the process (closing all fds) and starts a fresh process
  (which opens the port anew), there is no software path to prevent this
  reset. A hardware fix (capacitor between RESET and GND on the Arduino)
  would be required, but was explicitly not pursued per user decision.
  The fix implemented in this change (keeping the stream alive on transient
  errors) only prevents resets during *active use* — it provides value
  across long-running sessions where transient USB noise would otherwise
  cause a reconnect and reset, but cannot protect against container
  restarts.
