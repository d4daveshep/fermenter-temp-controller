## ADDED Requirements

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
