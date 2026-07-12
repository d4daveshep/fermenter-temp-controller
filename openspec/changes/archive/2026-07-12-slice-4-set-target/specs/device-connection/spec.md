## ADDED Requirements

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
