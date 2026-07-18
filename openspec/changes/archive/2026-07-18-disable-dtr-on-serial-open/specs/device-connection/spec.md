## ADDED Requirements

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
