# operational-health

## Purpose

The structured-logging and typed-error baseline used across the application
for observability and consistent error handling.

## Requirements

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

### Requirement: Liveness health check endpoint

The system SHALL expose an HTTP endpoint that reports whether the
application's core background processing (reading ingestion) is alive, so
that an external process (such as a container orchestrator) can determine
whether the application is functioning.

#### Scenario: Health check reports healthy while ingestion is running

- **WHEN** the health check endpoint is requested while the ingest loop is
  running and processing input
- **THEN** the response indicates the application is healthy, including a
  status field for the ingestion component

#### Scenario: Health check responds without requiring a prior reading

- **WHEN** the health check endpoint is requested before any reading has been
  observed
- **THEN** the response still succeeds, indicating the ingestion component's
  liveness rather than the presence of data
</content>
