## MODIFIED Requirements

### Requirement: Serial transport abstraction

The system SHALL access the serial device through a `SerialSource` abstraction
that supports reading a line and writing a target value, so that the transport
can be replaced (real hardware vs. mock) without changing dependent code.

#### Scenario: Read a line from the source

- **WHEN** a complete newline-terminated line is available from the serial source
- **THEN** the system receives that line as a UTF-8 string for processing

#### Scenario: Mock source drives the system without hardware

- **WHEN** the application runs in mock mode (no controller board attached)
- **THEN** a mock `SerialSource` supplies scripted lines and records any written
  target values, allowing the full ingest path to run

#### Scenario: Real hardware source drives the system

- **WHEN** the application runs with `MOCK_SERIAL=false` and a real controller
  board is attached over USB serial
- **THEN** a `tokio-serial`-backed `SerialSource` implementation reads real
  lines from the device and writes real target values to it through the same
  trait, exercising the full ingest path against physical hardware with no
  changes to dependent code

### Requirement: Serial framing and baud contract

The system SHALL communicate with the controller board using newline-delimited
JSON readings at 115200 baud, and SHALL write target temperatures as a `<float>`
framed string. This contract is fixed and device-agnostic: it applies equally
to the original Arduino Uno firmware and the replacement ESP32-C3 Rust firmware.

#### Scenario: Reading framing

- **WHEN** the controller board emits a reading
- **THEN** it arrives as a single newline-terminated JSON object that the system
  reads as one line

#### Scenario: Target write framing

- **WHEN** the system needs to send a target temperature to the device
- **THEN** it writes the value framed as `<value>` (e.g. `<19.5>`) to the serial
  port

#### Scenario: Zero is a valid target temperature

- **WHEN** the system sends `<0.0>` as the target temperature
- **THEN** the controller board accepts and applies `0.0` as the new target —
  zero is a valid lager temperature and SHALL NOT be silently ignored

## ADDED Requirements

### Requirement: Controller board communicates over native USB-Serial-JTAG

The ESP32-C3 controller board SHALL communicate with the host using its native
USB-Serial-JTAG peripheral, which enumerates as a CDC-ACM device on the host
(typically `/dev/ttyACM0` on Linux) and is indistinguishable from the previous
Arduino Uno's FTDI-bridged serial port from the host's perspective.

#### Scenario: Host reconnects after controller board reset

- **WHEN** the ESP32-C3 resets and the USB-Serial-JTAG peripheral re-enumerates
- **THEN** the host's existing serial reconnect-with-backoff behaviour (see
  "Serial connection resilience") handles the reconnection transparently,
  without any changes to `fermenter/` code

#### Scenario: Verified against real hardware

- **WHEN** the real serial connection to the attached ESP32-C3 board errors,
  closes, or is unplugged
- **THEN** the same reconnect-with-backoff behavior is exercised against actual
  hardware I/O, confirmed by `#[ignore]`'d hardware tests run manually via
  `cargo test -- --ignored`
