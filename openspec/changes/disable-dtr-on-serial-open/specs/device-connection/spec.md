## ADDED Requirements

### Requirement: Serial port opens without resetting the device

The system SHALL open the serial port without asserting DTR or RTS, so that
connecting or reconnecting to the port does not trigger a hardware reset of
an attached Arduino.

#### Scenario: Initial connection does not reset the device

- **WHEN** the system opens the serial port for the first time
- **THEN** DTR and RTS are not asserted, and the attached Arduino continues
  running without restarting

#### Scenario: Reconnection after a dropped connection does not reset the device

- **WHEN** the serial connection errors or closes and the system reopens the
  port (per the existing "Serial connection resilience" reconnect behavior)
- **THEN** DTR and RTS are not asserted on reopen, and the attached Arduino's
  in-progress state (including its lifetime min/max temperature tracking) is
  preserved rather than reset
</content>
