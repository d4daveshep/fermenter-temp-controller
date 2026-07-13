## MODIFIED Requirements

### Requirement: Compose orchestration with scoped device passthrough

The system SHALL be orchestrated via a single, repo-root Docker Compose
file alongside its Redis Time Series dependency, granting the application
container access to the serial device by passing through only that
specific device rather than running the container privileged.

#### Scenario: Serial device is passed through, not privileged

- **WHEN** the Compose stack starts the application container
- **THEN** the container is granted access to the configured serial device
  path via a scoped `devices:` mapping, and the container does not run in
  privileged mode

#### Scenario: Configuration is supplied via env file

- **WHEN** the Compose stack starts the application container
- **THEN** its configuration (serial port, Redis URL, ports, etc.) is
  supplied via an env file rather than being baked into the image

#### Scenario: Application depends on Redis being orchestrated together

- **WHEN** the Compose stack is started
- **THEN** the Redis Time Series service starts as part of the same stack
  and the application container is configured to depend on it

#### Scenario: A single compose file is the sole orchestration entry point

- **WHEN** an operator wants to run the full stack from the repository
- **THEN** there is exactly one Compose file, at the repository root,
  describing the complete deployable system — no separate or coexisting
  compose file for an alternate (e.g. previously Python/InfluxDB-based)
  stack exists in the repository
</content>
