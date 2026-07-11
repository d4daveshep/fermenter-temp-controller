## ADDED Requirements

### Requirement: Structured logging baseline

The system SHALL emit structured, level-filtered logs for significant events,
with the log level controllable via configuration, so operators can observe
behaviour and diagnose problems.

#### Scenario: Ingest events are logged

- **WHEN** the system reads a reading, skips a malformed line, or encounters a
  connection error
- **THEN** it emits a structured log event at an appropriate level with relevant
  context

#### Scenario: Log level is configurable

- **WHEN** the configured log level is changed
- **THEN** the verbosity of emitted logs changes accordingly

### Requirement: Typed error model

The system SHALL represent recoverable failures using a typed error model so
that error handling is explicit and consistent across modules.

#### Scenario: Errors carry typed context

- **WHEN** an operation such as parsing or serial I/O fails
- **THEN** it returns a typed error describing the failure category rather than
  an opaque or stringly-typed value
