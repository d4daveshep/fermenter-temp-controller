## MODIFIED Requirements

### Requirement: Dashboard rendering is read-only in this capability

The system SHALL NOT expose any HTTP endpoint under the dashboard page or the
live-polled status fragment that mutates controller state (target
temperature, brew identifier, or any other runtime setting); those two
routes only display current state. Mutating routes (such as the
target-setpoint form) are a distinct part of this capability and are exempt
from this requirement.

#### Scenario: No mutating routes are exposed by the dashboard or status fragment

- **WHEN** the web server's routes are enumerated
- **THEN** neither the dashboard page route nor the status fragment route
  accepts a request that changes target temperature, brew identifier, or any
  other runtime setting

## ADDED Requirements

### Requirement: Serve a target-setpoint form

The system SHALL serve an HTML form, at a distinct URL from the dashboard
page, that displays the current target temperature and accepts a submission
of a new target temperature.

#### Scenario: Form displays the current target temperature

- **WHEN** an operator requests the target-setpoint form
- **THEN** the response is an HTML form pre-filled with the current target
  temperature

#### Scenario: Valid submission updates the target and confirms

- **WHEN** an operator submits a valid new target temperature through the
  form
- **THEN** the system applies the new target temperature and the response
  confirms the update

#### Scenario: Invalid submission redisplays the form with an error

- **WHEN** an operator submits an invalid target temperature (non-numeric or
  outside the allowed range) through the form
- **THEN** the response redisplays the form with an error message and the
  current target temperature is left unchanged
