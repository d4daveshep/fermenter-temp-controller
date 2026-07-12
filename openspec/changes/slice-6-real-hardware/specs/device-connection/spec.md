## MODIFIED Requirements

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
</content>
