# temperature-monitoring

## Purpose

Ingesting serial lines into validated `Reading` values and exposing the latest
observed reading as current state.

## Requirements

### Requirement: Parse and validate temperature readings

The system SHALL parse each serial line into a strongly-typed `Reading` value
containing the required fields `target`, `average`, `min`, `max`, `ambient`
(numeric) and `action` (text). A `reason-code` field SHALL default to empty when
absent, and an unknown legacy `json-size` field SHALL be accepted but ignored.

#### Scenario: Valid reading parses successfully

- **WHEN** a line contains a JSON object with all required fields
- **THEN** the system produces a `Reading` with those values populated

#### Scenario: reason-code defaults when absent

- **WHEN** a reading omits `reason-code`
- **THEN** the resulting `Reading` has an empty `reason-code`

#### Scenario: legacy json-size is ignored

- **WHEN** a reading includes a `json-size` field
- **THEN** the system parses the reading and does not treat `json-size` as data
  to retain

### Requirement: Reject malformed readings without crashing

The system SHALL log malformed or incomplete serial lines at warning level and
skip them, continuing to process subsequent lines rather than terminating the
ingest loop.

#### Scenario: Malformed JSON line is skipped

- **WHEN** a serial line is not valid JSON or is missing a required field
- **THEN** the system logs a warning identifying the problem and continues
  reading the next line

### Requirement: Expose latest reading as current state

The system SHALL retain the most recently parsed `Reading` as the current
in-memory state, replacing it each time a new valid reading arrives.

#### Scenario: Current state updates on new reading

- **WHEN** a new valid reading is parsed
- **THEN** the current state reflects that reading as the latest observed value
</content>
