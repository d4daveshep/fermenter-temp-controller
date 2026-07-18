# device-connection

## Purpose

The serial transport contract between the controller and the Arduino: baud
rate, newline-delimited JSON framing, the `SerialSource` abstraction, and
reconnect/backoff behaviour.

## Requirements

### Requirement: Serial transport abstraction

The system SHALL access the serial device through a `SerialSource` abstraction
that supports reading a line and writing a target value, so that the transport
can be replaced (real hardware vs. mock) without changing dependent code.

#### Scenario: Read a line from the source

- **WHEN** a complete newline-terminated line is available from the serial source
- **THEN** the system receives that line as a UTF-8 string for processing

#### Scenario: Mock source drives the system without hardware

- **WHEN** the application runs in mock mode (no Arduino attached)
- **THEN** a mock `SerialSource` supplies scripted lines and records any written
  target values, allowing the full ingest path to run

#### Scenario: Real hardware source drives the system

- **WHEN** the application runs with `MOCK_SERIAL=false` and a real Arduino is
  attached over USB serial
- **THEN** a `tokio-serial`-backed `SerialSource` implementation reads real
  lines from the device and writes real target values to it through the same
  trait, exercising the full ingest path against physical hardware with no
  changes to dependent code

### Requirement: Serial framing and baud contract

The system SHALL communicate with the Arduino using newline-delimited JSON
readings at 115200 baud, and SHALL write target temperatures as a `<float>`
framed string, matching the fixed firmware contract.

#### Scenario: Reading framing

- **WHEN** the Arduino emits a reading
- **THEN** it arrives as a single newline-terminated JSON object that the system
  reads as one line

#### Scenario: Target write framing

- **WHEN** the system needs to send a target temperature to the device
- **THEN** it writes the value framed as `<value>` (e.g. `<19.5>`) to the serial
  port

### Requirement: Serial connection resilience

The system SHALL recover from serial connection failures by reconnecting with
exponential backoff, so transient device disconnects do not permanently stop
ingestion.

#### Scenario: Reconnect after a dropped connection

- **WHEN** the serial connection errors or closes
- **THEN** the system logs the failure and retries opening the connection with
  exponentially increasing, capped backoff delays until it succeeds

#### Scenario: Verified against real hardware

- **WHEN** the real serial connection to an attached Arduino errors, closes, or
  is unplugged
- **THEN** the same reconnect-with-backoff behavior is exercised against
  actual hardware I/O (not just the mock), confirmed by `#[ignore]`'d
  hardware tests run manually via `cargo test -- --ignored`

### Requirement: Write the target only when it differs from the device-reported value

The system SHALL write a target temperature to the device only when the
desired target temperature differs from the target most recently reported by
the device, avoiding redundant writes on every reading.

#### Scenario: Write occurs when the desired target differs

- **WHEN** the desired target temperature differs from the target most
  recently reported by the device
- **THEN** the system writes the desired target temperature to the device
  using the `<value>` framing

#### Scenario: No write occurs when already in sync

- **WHEN** the desired target temperature matches the target most recently
  reported by the device
- **THEN** the system does not write to the device

### Requirement: Serial port stays open across transient errors

The system SHALL keep the serial port open when a transient read, write, or
flush error occurs, so that the attached Arduino is not reset by a
DTR-triggered port reopen during normal operation. The port SHALL only be
closed and reopened when the stream reaches EOF (device physically
disconnected).

#### Scenario: Read error does not reopen the port

- **WHEN** a `read_line` call returns an I/O error
- **THEN** the error is returned to the caller and the stream is kept alive

#### Scenario: Write error does not reopen the port

- **WHEN** a `write_target` call returns a write or flush error
- **THEN** the error is returned to the caller and the stream is kept alive

#### Scenario: EOF triggers reconnect

- **WHEN** a `read_line` call returns EOF (the device stream closed)
- **THEN** the stream is dropped and the next call reopens the port with
  exponential backoff, per the existing "Serial connection resilience"
  requirement

### Requirement: Serial port opens without asserting DTR (best-effort)

The system SHALL attempt to open the serial port without asserting DTR, by
calling `.dtr_on_open(false)` on the port builder. On Linux this cannot
prevent a brief DTR pulse during the kernel's `open()` syscall (a USB CDC
ACM protocol requirement), but it clears DTR immediately afterward and
provides correct DTR state on platforms where the `serialport` crate has
full control.

#### Scenario: Builder configuration clears DTR after open

- **WHEN** the serial port is opened
- **THEN** DTR is de-asserted via the builder's `.dtr_on_open(false)` after
  the port is opened, independent of the kernel-level pulse during `open()`
</content>
