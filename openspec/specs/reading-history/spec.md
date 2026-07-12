# reading-history

## Purpose

Persisting temperature readings and controller state to time-series storage,
and retrieving that persisted data for query and startup rehydration.

## Requirements

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

### Requirement: Idempotent, retention-bounded series creation

The system SHALL create time-series storage lazily on first write for a given
brew/field combination, applying the configured retention period, and SHALL
treat an attempt to create an already-existing series as successful rather
than as an error.

#### Scenario: First write creates the series

- **WHEN** a reading is written for a brew/field combination with no existing
  series
- **THEN** the system creates the series with the configured retention period
  before writing the value

#### Scenario: Repeated creation is tolerated

- **WHEN** the system attempts to create a series that already exists
- **THEN** it proceeds without treating the existing series as an error

### Requirement: Query the most recent persisted reading

The system SHALL support retrieving the most recently persisted `Reading` for
a given brew identifier.

#### Scenario: Most recent reading is returned

- **WHEN** the system queries for the most recent reading of a brew that has
  persisted data
- **THEN** it returns the last reading written for that brew

#### Scenario: No data returns nothing

- **WHEN** the system queries for the most recent reading of a brew with no
  persisted data
- **THEN** it returns no reading, without error

### Requirement: Persist and restore controller state across restarts

The system SHALL save the current target temperature and brew identifier to
durable storage whenever they are established, and SHALL be able to load the
most recently saved values back on startup.

#### Scenario: Saved state round-trips

- **WHEN** controller state (target temperature and brew identifier) is saved
  and then loaded
- **THEN** the loaded values match what was saved

#### Scenario: Loading with no prior state returns nothing

- **WHEN** the system loads controller state before any state has ever been
  saved
- **THEN** it returns no state, without error

### Requirement: Time-series storage connection resilience

The system SHALL automatically reconnect to time-series storage after a
connection failure, using a backoff strategy, without requiring the
application to restart.

#### Scenario: Reconnect after a dropped connection

- **WHEN** the connection to time-series storage is lost
- **THEN** the system automatically attempts to reconnect with backoff and
  resumes writes once reconnected
</content>
