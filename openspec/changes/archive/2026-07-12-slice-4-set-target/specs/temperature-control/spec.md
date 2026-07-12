## ADDED Requirements

### Requirement: Accept a new target temperature setpoint

The system SHALL accept a new target temperature submitted by an operator,
validating that it is a finite number within a configured sane range, and
SHALL hold the accepted value as the current, authoritative target
temperature for subsequent reconciliation to the device.

#### Scenario: Valid target is accepted

- **WHEN** an operator submits a target temperature that is a finite number
  within the allowed range
- **THEN** the system accepts it as the current target temperature

#### Scenario: Non-numeric target is rejected

- **WHEN** an operator submits a target temperature value that does not
  parse as a number
- **THEN** the system rejects the submission, reports the error, and leaves
  the current target temperature unchanged

#### Scenario: Out-of-range target is rejected

- **WHEN** an operator submits a target temperature outside the configured
  allowed range
- **THEN** the system rejects the submission, reports the error, and leaves
  the current target temperature unchanged

### Requirement: Reconcile the target temperature to the device

The system SHALL compare the current target temperature against the target
temperature most recently reported by the device, and SHALL write the
current target temperature to the device when the two differ.

#### Scenario: Reconcile writes a changed target

- **WHEN** the current target temperature differs from the target most
  recently reported by the device
- **THEN** the system writes the current target temperature to the device

#### Scenario: Reconcile is a no-op when already in sync

- **WHEN** the current target temperature matches the target most recently
  reported by the device
- **THEN** the system does not write to the device

### Requirement: Persist and restore the target temperature across restarts

The system SHALL persist the current target temperature whenever it is
changed, and SHALL restore the most recently persisted target temperature as
the initial current target temperature on startup, falling back to a
configured default when nothing has been persisted.

#### Scenario: Accepted target change is persisted

- **WHEN** the system accepts a new target temperature from an operator
- **THEN** it persists that value to durable storage

#### Scenario: Startup restores a previously persisted target

- **WHEN** the system starts and durable storage holds a previously
  persisted target temperature
- **THEN** the system uses that persisted value as its initial current
  target temperature

#### Scenario: Startup falls back to the configured default

- **WHEN** the system starts and durable storage holds no previously
  persisted target temperature
- **THEN** the system uses the configured default target temperature as its
  initial current target temperature
