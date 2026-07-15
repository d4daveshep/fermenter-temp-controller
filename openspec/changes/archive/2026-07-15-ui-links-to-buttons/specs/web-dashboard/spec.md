## MODIFIED Requirements

### Requirement: Serve a target-setpoint form

The system SHALL serve an HTML form, at a distinct URL from the dashboard
page, that displays the current target temperature and accepts a submission
of a new target temperature. On successful submission, the system SHALL
redirect the browser back to the dashboard page.

#### Scenario: Form displays the current target temperature

- **WHEN** an operator requests the target-setpoint form
- **THEN** the response is an HTML form pre-filled with the current target
  temperature

#### Scenario: Valid submission updates the target and redirects to dashboard

- **WHEN** an operator submits a valid new target temperature through the
  form
- **THEN** the system applies the new target temperature and redirects the
  browser to the dashboard page

#### Scenario: Invalid submission redisplays the form with an error

- **WHEN** an operator submits an invalid target temperature (non-numeric or
  outside the allowed range) through the form
- **THEN** the response redisplays the form with an error message and the
  current target temperature is left unchanged

### Requirement: Serve a brew-identifier form

The system SHALL serve an HTML form, at a distinct URL from the dashboard
page, that displays the current brew identifier and accepts a submission of
a new brew identifier. On successful submission, the system SHALL redirect
the browser back to the dashboard page.

#### Scenario: Form displays the current brew identifier

- **WHEN** an operator requests the brew-identifier form
- **THEN** the response is an HTML form pre-filled with the current brew
  identifier

#### Scenario: Valid submission updates the brew identifier and redirects to dashboard

- **WHEN** an operator submits a valid new brew identifier through the form
- **THEN** the system applies the new brew identifier and redirects the
  browser to the dashboard page

#### Scenario: Invalid submission redisplays the form with an error

- **WHEN** an operator submits an invalid brew identifier (empty or
  whitespace-only) through the form
- **THEN** the response redisplays the form with an error message and the
  current brew identifier is left unchanged
