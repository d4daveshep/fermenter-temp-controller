## MODIFIED Requirements

### Requirement: Persist readings to time-series storage

The system SHALL write every successfully parsed `Reading` to time-series
storage, tagged by the brew identifier that is currently active at the
moment the reading is written, in addition to updating in-memory current
state. The active brew identifier can change at runtime; a change SHALL only
affect readings written after the change, and SHALL NOT alter the brew
identifier already recorded against readings persisted before it.

#### Scenario: Valid reading is persisted

- **WHEN** a serial line parses into a valid `Reading`
- **THEN** the system writes that reading to time-series storage tagged with
  the brew identifier that is currently active

#### Scenario: Persistence failure does not crash the ingest loop

- **WHEN** a write to time-series storage fails
- **THEN** the system logs the failure and continues processing subsequent
  serial lines rather than terminating

#### Scenario: Readings persisted before a brew switch keep their original tag

- **WHEN** the active brew identifier changes at runtime after readings have
  already been persisted under a previous identifier
- **THEN** those previously persisted readings remain queryable under the
  previous identifier, unaffected by the switch
