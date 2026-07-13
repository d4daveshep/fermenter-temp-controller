## MODIFIED Requirements

### Requirement: Expose latest reading as current state

The system SHALL retain the most recently parsed `Reading` as the current
in-memory state, replacing it each time a new valid reading arrives. On
startup, if a previously persisted reading exists for the current brew
identifier, the system SHALL initialise current state from that persisted
reading instead of starting with no reading.

#### Scenario: Current state updates on new reading

- **WHEN** a new valid reading is parsed
- **THEN** the current state reflects that reading as the latest observed
  value

#### Scenario: Current state rehydrates from persisted storage on startup

- **WHEN** the application starts and a persisted reading exists for the
  current brew identifier
- **THEN** current state is initialised from that persisted reading before
  any new serial line has been read

#### Scenario: Current state starts empty when nothing is persisted

- **WHEN** the application starts and no persisted reading exists for the
  current brew identifier
- **THEN** current state starts with no reading, as before
