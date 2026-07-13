# brew-session

## Purpose

Managing the brew identifier lifecycle: accepting an operator-submitted
relabel at runtime, holding it as the current authoritative identifier,
tagging subsequent readings and persisted state with it, persisting and
restoring it across restarts, and rehydrating current state when the active
brew changes.

## Requirements

### Requirement: Accept a new brew identifier at runtime

The system SHALL accept a new brew identifier submitted by an operator,
validating that it is non-empty after trimming surrounding whitespace, and
SHALL hold the accepted value as the current, authoritative brew identifier
used to tag subsequent readings and persisted state, replacing the previous
identifier.

#### Scenario: Valid brew identifier is accepted

- **WHEN** an operator submits a non-empty brew identifier (after trimming
  surrounding whitespace)
- **THEN** the system accepts it as the current brew identifier

#### Scenario: Empty brew identifier is rejected

- **WHEN** an operator submits a brew identifier that is empty or contains
  only whitespace
- **THEN** the system rejects the submission, reports the error, and leaves
  the current brew identifier unchanged

### Requirement: Tag subsequent readings with the newly active brew identifier

The system SHALL tag every reading ingested after a brew-identifier change
with the newly active identifier when persisting it, without altering how
readings persisted before the change were tagged.

#### Scenario: Reading ingested after a switch is tagged with the new identifier

- **WHEN** the active brew identifier is changed and a subsequent reading is
  successfully parsed
- **THEN** the system persists that reading tagged with the new brew
  identifier

### Requirement: Rehydrate current state when the active brew changes

The system SHALL immediately refresh the current in-memory state to reflect
the newly active brew identifier's most recently persisted reading when an
operator changes the active brew, rather than continuing to display the
previous brew's last reading mislabeled under the new identifier.

#### Scenario: Switching to a brew with existing history shows that history immediately

- **WHEN** an operator switches the active brew identifier to one with a
  previously persisted reading
- **THEN** the current state immediately reflects that persisted reading,
  before any new serial line has been read under the new identifier

#### Scenario: Switching to a brew with no history shows no data immediately

- **WHEN** an operator switches the active brew identifier to one with no
  previously persisted reading
- **THEN** the current state immediately shows no reading available, rather
  than retaining the previous brew's last reading

### Requirement: Persist and restore the brew identifier across restarts

The system SHALL persist the current brew identifier whenever it is changed,
and SHALL restore the most recently persisted brew identifier as the initial
current brew identifier on startup, falling back to a configured default when
nothing has been persisted.

#### Scenario: Accepted brew change is persisted

- **WHEN** the system accepts a new brew identifier from an operator
- **THEN** it persists that value to durable storage

#### Scenario: Startup restores a previously persisted brew identifier

- **WHEN** the system starts and durable storage holds a previously
  persisted brew identifier
- **THEN** the system uses that persisted value as its initial current brew
  identifier

#### Scenario: Startup falls back to the configured default

- **WHEN** the system starts and durable storage holds no previously
  persisted brew identifier
- **THEN** the system uses the configured default brew identifier as its
  initial current brew identifier
