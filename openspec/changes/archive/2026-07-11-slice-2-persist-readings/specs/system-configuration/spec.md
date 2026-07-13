## MODIFIED Requirements

### Requirement: Load configuration from environment

The system SHALL load its configuration from environment variables at startup
into a typed configuration value, supporting at minimum the serial port,
serial baud rate, a mock-serial toggle, the log level, the time-series storage
connection URL, the time-series retention period, and the default brew
identifier.

#### Scenario: Configuration loaded from environment

- **WHEN** the application starts with the required environment variables set
- **THEN** it parses them into a typed configuration used by the rest of the
  application

#### Scenario: Mock-serial toggle selects the source

- **WHEN** the mock-serial environment variable is enabled
- **THEN** the application uses the mock serial source instead of real
  hardware

#### Scenario: Time-series storage configuration is loaded

- **WHEN** the application starts with the time-series storage connection URL,
  retention period, and default brew identifier set in the environment
- **THEN** it parses them into the typed configuration alongside the serial
  and logging settings
